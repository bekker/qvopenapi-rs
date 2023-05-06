use log::*;
use std::{
    collections::VecDeque,
    future::Future,
    pin::Pin,
    sync::{Arc, Mutex, RwLock},
    task::{Context, Poll, Waker},
};

use crate::*;

type WmcaRequestType = dyn WmcaRequest + Send + Sync;
type WmcaResponseType = dyn Send + Sync;

lazy_static! {
    static ref REQUEST_QUEUE_LOCK: RwLock<VecDeque<Arc<ResponseFutureInner<WmcaResponseType>>>> =
        RwLock::new(VecDeque::new());
    static ref ACTIVE_REQUEST_LOCK: Mutex<Option<Arc<ResponseFutureInner<WmcaResponseType>>>> =
        Mutex::new(None);
    static ref WAKERS_LOCK: Mutex<VecDeque<Waker>> = Mutex::new(VecDeque::new());
}

pub struct ResponseFutureInner<T: ?Sized> {
    pub request: Arc<WmcaRequestType>,
    pub response: Mutex<Option<Result<Arc<T>, QvOpenApiError>>>,
}

pub struct ResponseFuture<T: ?Sized> {
    pub inner: Arc<ResponseFutureInner<T>>,
}

impl<T> Future for ResponseFuture<T> {
    type Output = Result<Arc<T>, QvOpenApiError>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        //debug!("polling...");
        let res = self.inner.response.lock().unwrap();

        match res.as_ref() {
            Some(result) => match result {
                Ok(v) => Poll::Ready(Ok(v.clone())),
                Err(e) => Poll::Ready(Err(e.clone())),
            },
            None => {
                let mut wakers = WAKERS_LOCK.lock().unwrap();
                wakers.push_back(cx.waker().clone());
                Poll::Pending
            }
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
    fn before_post(&self) -> Result<(), QvOpenApiError> {
        if wmca_lib::is_connected()? {
            return Err(QvOpenApiError::AlreadyConnectedError)
        }

        Ok(())
    }
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
    fn before_post(&self) -> Result<(), QvOpenApiError> {
        wmca_lib::assert_connected()
    }
    fn call_lib(&self) -> Result<(), QvOpenApiError> {
        wmca_lib::query(&self.tr_code, &self.input, self.account_index)
    }
}

pub trait WmcaRequest {
    fn before_post(&self) -> Result<(), QvOpenApiError>;
    fn call_lib(&self) -> Result<(), QvOpenApiError>;
}

pub fn post_request<T>(req: Arc<WmcaRequestType>) -> ResponseFuture<T> {
    debug!("post_request");
    let ret = do_post_request(req.clone());
    match ret {
        Ok(f) => ResponseFuture { inner: f },
        Err(e) => ResponseFuture {
            inner: Arc::new(ResponseFutureInner {
                request: req,
                response: Mutex::new(Some(Err(e))),
            }),
        },
    }
}

fn do_post_request<T>(
    req: Arc<WmcaRequestType>,
) -> Result<Arc<ResponseFutureInner<T>>, QvOpenApiError> {
    debug!("do_post_request");

    let future: Arc<ResponseFutureInner<T>> = Arc::new(ResponseFutureInner {
        request: req,
        response: Mutex::new(None),
    });
    unsafe {
        let mut queue = REQUEST_QUEUE_LOCK.write().unwrap();
        queue.push_back(std::mem::transmute(future.clone()));
    }
    let mut current_req = ACTIVE_REQUEST_LOCK.lock().unwrap();
    activate_next_request_if_available(&mut current_req)?;

    Ok(future)
}

fn activate_next_request_if_available(
    current_req: &mut Option<Arc<ResponseFutureInner<WmcaResponseType>>>,
) -> Result<(), QvOpenApiError> {
    debug!("activate_next_request_if_available: Trying to get lock...");
    if current_req.is_none() {
        let mut queue = REQUEST_QUEUE_LOCK.write().unwrap();
        let req = queue.pop_front();

        if req.is_some() {
            req.as_ref().unwrap().request.before_post()?;

            *current_req = req;

            return window_mgr::post_message(
                message::WM_CUSTOMEVENT,
                message::CA_COMMAND.try_into().unwrap(),
                0,
            );
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

pub fn end_active_request(
    result: Result<Arc<WmcaResponseType>, QvOpenApiError>,
) -> Result<(), QvOpenApiError> {
    debug!("end_active_request");
    let mut current_req = ACTIVE_REQUEST_LOCK.lock().unwrap();

    if current_req.is_none() {
        error!("Tried to register result, but no active request found");
        return Ok(());
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
