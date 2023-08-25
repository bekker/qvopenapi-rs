use std::{future::Future, pin::Pin, task::{Context, Poll, Waker}, sync::{Arc, Mutex}, any::Any};
use qvopenapi::*;

pub struct ResponseFuture {
    inner: Arc<Mutex<ResponseFutureInner>>
}

impl Clone for ResponseFuture {
    fn clone(&self) -> ResponseFuture {
        ResponseFuture { inner: self.inner.clone() }
    }
}

struct ResponseFutureInner {
    pub response: Option<Result<Arc<dyn Any + Send + Sync>, QvOpenApiError>>,
    pub waker: Option<Waker>,
}

impl ResponseFuture {
    pub fn new() -> ResponseFuture {
        ResponseFuture { inner: Arc::new(Mutex::new(ResponseFutureInner::new())) }
    }

    pub fn resolve(&self, res: Result<Arc<dyn Any + Send + Sync>, QvOpenApiError>) {
        let mut locked = self.inner.lock().unwrap();
        locked.response = Some(res);
        if locked.waker.is_some() {
            locked.waker.as_ref().unwrap().wake_by_ref();
        }
    }
}

impl ResponseFutureInner {
    fn new() -> ResponseFutureInner {
        ResponseFutureInner {
            response: None,
            waker: None
        }
    }
}

impl Future for ResponseFuture {
    type Output = Result<Arc<dyn Any + Send + Sync>, QvOpenApiError>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut locked = self.inner.lock().unwrap();

        match locked.response.as_ref() {
            Some(result) => Poll::Ready(result.clone()),
            None => {
                locked.waker = Some(cx.waker().clone());
                Poll::Pending
            }
        }
    }
}
