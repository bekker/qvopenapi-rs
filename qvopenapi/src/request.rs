use log::*;
use std::{
    collections::VecDeque,
    sync::{Arc, RwLock, Mutex}, future::Future, pin::Pin, task::{Context, Poll, Waker},
};

use crate::{*};

type WmcaRequestType = dyn WmcaRequest + Send + Sync;
type WmcaResponseType = dyn WmcaResponse + Send + Sync;

lazy_static! {
    static ref REQUEST_QUEUE_LOCK: RwLock<VecDeque<Arc<ResponseFutureInner>>> = RwLock::new(VecDeque::new());
    static ref ACTIVE_REQUEST_LOCK: Mutex<Option<Arc<ResponseFutureInner>>> = Mutex::new(None);
    static ref WAKERS_LOCK: Mutex<VecDeque<Waker>> = Mutex::new(VecDeque::new());
}

pub struct ResponseFutureInner {
    pub request: Arc<WmcaRequestType>,
    pub response: Mutex<Option<Result<Arc<WmcaResponseType>, QvOpenApiError>>>,
}

pub struct ResponseFuture {
    pub inner: Arc<ResponseFutureInner>
}

impl Future for ResponseFuture {
    type Output = Result<Arc<WmcaResponseType>, QvOpenApiError>;
 
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        //debug!("polling...");
        let res = self.inner.response.lock().unwrap();
        
        match res.as_ref() {
            Some(result) => 
                match result {
                    Ok(v) => Poll::Ready(Ok(v.clone())),
                    Err(e) => Poll::Ready(Err(e.clone()))
                },
            None => {
                let mut wakers = WAKERS_LOCK.lock().unwrap();
                wakers.push_back(cx.waker().clone());
                Poll::Pending
            },
        }
    }
}

pub struct ConnectRequest {
    pub account_type: AccountType,
    pub id: String,
    pub password: String,
    pub cert_password: String,
}

impl WmcaRequest for ConnectRequest {
    fn call_lib(&self) -> Result<(), QvOpenApiError> {
        wmca_lib::connect(
            self.account_type,
            &self.id,
            &self.password,
            &self.cert_password,
        )
    }
}

pub struct QueryRequest {
    pub tr_code: String,
    pub input: String,
    pub account_index: i32,
}

impl WmcaRequest for QueryRequest {
    fn call_lib(&self) -> Result<(), QvOpenApiError> {
        wmca_lib::query(&self.tr_code, &self.input, self.account_index)
    }
}

pub trait WmcaRequest {
    fn call_lib(&self) -> Result<(), QvOpenApiError>;
}

pub trait WmcaResponse {

}

pub fn post_request(req: Arc<WmcaRequestType>) -> ResponseFuture {
    debug!("post_request");
    let ret = do_post_request(req.clone());
    match ret {
        Ok(f) => ResponseFuture { inner: f },
        Err(e) => ResponseFuture {
            inner: Arc::new(ResponseFutureInner {
                request: req,
                response: Mutex::new(Some(Err(e)))
            })
        }
    }
}

fn do_post_request(req: Arc<WmcaRequestType>) -> Result<Arc<ResponseFutureInner>, QvOpenApiError> {
    debug!("do_post_request");

    let future = Arc::new(ResponseFutureInner {
        request: req,
        response: Mutex::new(None),
    });
    {
        let mut queue = REQUEST_QUEUE_LOCK.write().unwrap();
        queue.push_back(future.clone());
    }
    let mut current_req = ACTIVE_REQUEST_LOCK.lock().unwrap();
    activate_next_request_if_available(&mut current_req)?;

    Ok(future)
}

fn activate_next_request_if_available(current_req: &mut Option<Arc<ResponseFutureInner>>) -> Result<(), QvOpenApiError> {
    debug!("activate_next_request_if_available: Trying to get lock...");
    if current_req.is_none() {
        let mut queue = REQUEST_QUEUE_LOCK.write().unwrap();
        let req = queue.pop_front();

        if req.is_some() {
            *current_req = req;

            return window_mgr::post_message(
                message::WM_CUSTOMEVENT,
                message::CA_COMMAND.try_into().unwrap(),
                0,
            )
        } else {
            debug!("activate_next_request_if_available: No active req found");
        }
    } else {
        debug!("activate_next_request_if_available: There is ongoing request, skipping");
    }

    Ok(())
}

pub fn execute_active_request() -> Result<(), QvOpenApiError> {
    debug!("execute_active_request");
    let current_req = ACTIVE_REQUEST_LOCK.lock().unwrap();

    if current_req.is_none() {
        error!("Tried to execute request, but no active request found");
        return Ok(());
    }

    current_req.as_ref().unwrap().request.call_lib()
}

pub fn end_active_request(result: Result<Arc<WmcaResponseType>, QvOpenApiError>) -> Result<(), QvOpenApiError> {
    debug!("end_active_request");
    let mut current_req = ACTIVE_REQUEST_LOCK.lock().unwrap();

    if current_req.is_none() {
        error!("Tried to register result, but no active request found");
        return Ok(())
    }

    {
        let mut res = current_req.as_mut().unwrap().response.lock().unwrap();
        *res = Some(result);
    }

    flush_wakers();

    *current_req = None;
    activate_next_request_if_available(&mut current_req)
}

fn flush_wakers() {
    let mut wakers = WAKERS_LOCK.lock().unwrap();
    while let Some(waker) = wakers.pop_front() {
        waker.wake();
    }
}
