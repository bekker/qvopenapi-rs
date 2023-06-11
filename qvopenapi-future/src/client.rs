use std::{sync::{Arc, Mutex}, collections::HashMap, any::Any};

use qvopenapi::*;

use crate::future::ResponseFuture;

const CONNECT_TR_INDEX: i32 = 1;

pub struct AsyncQvOpenApiClient {
    delegate: SimpleQvOpenApiClient,
    future_map_lock: Mutex<HashMap<i32, ResponseFuture>>,
}

impl qvopenapi::QvOpenApiClient for AsyncQvOpenApiClient {
    fn get_handler(&self) -> Arc<QvOpenApiClientMessageHandler> {
        self.delegate.get_handler()
    }
}

impl AsyncQvOpenApiClient {
    fn new() -> AsyncQvOpenApiClient {
        let mut client = SimpleQvOpenApiClient::new();
        client.on_connect(|res| {
            // TODO
        });

        AsyncQvOpenApiClient {
            delegate: client,
            future_map_lock: Mutex::new(HashMap::new()),
        }
    }

    pub async fn connect(
        &self,
        new_hwnd: isize,
        account_type: qvopenapi::AccountType,
        id: &str,
        password: &str,
        cert_password: &str,
    ) -> Result<Arc<ConnectResponse>, QvOpenApiError> {
        let mut future_map = self.future_map_lock.lock().unwrap();
        if future_map.contains_key(&CONNECT_TR_INDEX) {
            return Err(QvOpenApiError::TransactionPoolFullError);
        }

        let future: ResponseFuture = ResponseFuture::new();
        future_map.insert(CONNECT_TR_INDEX, future.clone());
        match self.delegate.connect(new_hwnd, account_type, id, password, cert_password) {
            Ok(_) => downcast_future(future).await,
            Err(e) => {
                future_map.remove(&CONNECT_TR_INDEX);
                Err(e)
            }
        }
    }

    pub fn get_balance(
        &self,
        tr_index: i32,
        account_index: i32,
        balance_type: char,
    ) -> Result<(), QvOpenApiError> {
        self.delegate.get_balance(tr_index, account_index, balance_type)
    }
}

async fn downcast_future<T: Any + Send + Sync>(future: ResponseFuture) -> Result<Arc<T>, QvOpenApiError> {
    match future.await {
        Ok(v) => Ok(v.downcast::<T>().unwrap()),
        Err(e) => Err(e)
    }
}
