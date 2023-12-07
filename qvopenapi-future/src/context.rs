use std::{sync::{Mutex, Arc}, time::Instant, task::{Waker, Context, Poll}, collections::HashMap, future::Future, pin::Pin};

use log::error;
use qvopenapi::{error::*, models::*};
use serde_json::{Value, json};


pub struct TrContext {
    pub tr_index: i32,
    pub tr_type: TrType,
    pub request_timestamp: Instant,
    pub status: Mutex<TrContextStatus>,
}

impl TrContext {
    pub fn new(tr_index: i32, tr_type: TrType) -> TrContext {
        TrContext {
            tr_index,
            tr_type,
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
            result_map.insert("connect_info".into(), json!(res));
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
        self.on_custom_error(QvOpenApiError::NotConnectedError)
    }

    pub fn on_message(&self, msg: MessageResponse) -> bool {
        let mut status = self.status.lock().unwrap();
        status.messages.push(msg.clone());

        if msg.msg.contains("잘못된 계좌 인덱스 번호") {
            status.error_type = Some(QvOpenApiError::BadRequestError { message: "잘못된 계좌 인덱스 번호".into() });
            status.set_done();
            return true;
        }
        return false;
    }

    pub fn on_error_response(&self, err: ErrorResponse) -> bool {
        let mut status = self.status.lock().unwrap();
        status.errors.push(err);
        return false;
    }

    pub fn on_timeout(&self) -> bool {
        error!("Request timed out (tr_index: {}, tr_type: {:?})", self.tr_index, self.tr_type);
        self.on_custom_error(QvOpenApiError::RequestTimeoutError)
    }

    pub fn on_custom_error(&self, err: QvOpenApiError) -> bool {
        let mut status = self.status.lock().unwrap();
        status.error_type = Some(err);
        status.set_done();
        return true;
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
    errors: Vec<ErrorResponse>,
    error_type: Option<QvOpenApiError>,
}

impl TrContextStatus {
    fn new() -> TrContextStatus {
        TrContextStatus { output: Value::Null, is_done: false, waker: None, result: HashMap::new(), messages: Vec::new(), errors: Vec::new(), error_type: None }
    }

    fn set_done(&mut self) {
        self.is_done = true;
        self.output = json!({
            "result": self.result,
            "messages": self.messages,
            "error_type": self.error_type,
            "errors": self.errors,
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
