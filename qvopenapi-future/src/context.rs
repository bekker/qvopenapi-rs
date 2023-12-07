use std::{sync::{Mutex, Arc}, time::Instant, task::{Waker, Context, Poll}, collections::HashMap, future::Future, pin::Pin};

use log::error;
use qvopenapi::{error::*, models::*};
use serde_json::{Value, json};


pub struct TrContext {
    pub tr_type: TrType,
    pub request_timestamp: Instant,
    pub status: Mutex<TrContextStatus>,
}

impl TrContext {
    pub fn new(tr_type: TrType) -> TrContext {
        TrContext {
            tr_type: tr_type,
            request_timestamp: Instant::now(),
            status: Mutex::new(TrContextStatus::new()),
        }
    }

    pub fn on_connect(&self, res: &ConnectResponse) -> bool {
        if !matches!(self.tr_type, TrType::CONNECT) {
            error!("Expected tr type CONNECT, but {:?} found", self.tr_type);
            return false;
        }

        let mut status = self.status.lock().unwrap();

        {
            let result_map = &mut status.result;
            result_map.insert("LOGIN_BLOCK".into(), json!(res));
        }

        status.set_done();
        return true;
    }

    pub fn on_data(&self, res: &DataResponse) -> bool {
        if !matches!(self.tr_type, TrType::QUERY) {
            error!("Expected tr type QUERY, but {:?} found", self.tr_type);
            return false;
        }

        let mut status = self.status.lock().unwrap();
        let result_map = &mut status.result;
        result_map.insert(res.block_name.clone(), res.block_data.clone());
        return false;
    }

    pub fn on_complete(&self) -> bool {
        if !matches!(self.tr_type, TrType::QUERY) {
            error!("Expected tr type QUERY, but {:?} found", self.tr_type);
            return false;
        }
    
        let mut status = self.status.lock().unwrap();
        status.set_done();
        return true;
    }

    pub fn on_disconnect(&self) -> bool {
        let mut status = self.status.lock().unwrap();
        status.messages.push(MessageResponse {
            tr_index: -1,
            msg_code: "-----".into(),
            msg: "Disconnected".into(),
        });
        status.set_done();
        return true;
    }

    pub fn on_message(&self, msg: MessageResponse) -> bool {
        let mut status = self.status.lock().unwrap();
        status.messages.push(msg);
        return false;
    }
}

#[derive(Debug, Clone)]
pub enum TrType {
    CONNECT,
    QUERY
}

pub struct TrContextStatus {
    output: Value,
    is_done: bool,
    waker: Option<Waker>,
    result: HashMap<String, Value>,
    messages: Vec<MessageResponse>,
}

impl TrContextStatus {
    fn new() -> TrContextStatus {
        TrContextStatus { output: Value::Null, is_done: false, waker: None, result: HashMap::new(), messages: Vec::new() }
    }

    fn set_done(&mut self) {
        self.is_done = true;
        self.output = json!({
            "result": self.result,
            "messages": self.messages,
        });
        match &self.waker {
            Some(waker) => {
                waker.wake_by_ref();
            }
            None => {}
        }
        self.waker = None;
    }
}

pub struct TrFuture {
    context: Result<Arc<TrContext>, QvOpenApiError>,
}

impl Future for TrFuture {
    type Output = Result<Value, QvOpenApiError>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output>
    {
        match &self.context {
            Ok(context) => {
                let mut status = context.status.lock().unwrap();
                Self::poll_for_status(&mut status, cx)
            },
            Err(e) => {
                Poll::Ready(Err(e.clone()))
            }
        }
    }
}

impl TrFuture {
    pub fn new(context: Result<Arc<TrContext>, QvOpenApiError>) -> TrFuture {
        TrFuture { context: context }
    }

    fn poll_for_status(status: &mut TrContextStatus, cx: &mut Context<'_>) -> Poll<Result<Value, QvOpenApiError>>
    {
        if status.is_done {
            Poll::Ready(Ok(status.output.clone()))
        } else {
            if status.waker.as_ref().map_or(true, |w| !w.will_wake(cx.waker())) {
                status.waker = Some(cx.waker().clone());
            }
            Poll::Pending
        }
    }
}
