use std::{sync::{Arc, Mutex, RwLock}, collections::HashMap, any::Any};

use log::*;
use qvopenapi::*;

use crate::future::ResponseFuture;

const CONNECT_TR_INDEX: i32 = 1;

pub struct FutureQvOpenApiClient {
    delegate: Box<dyn AbstractQvOpenApiClient>,
    future_map_lock: Arc<Mutex<HashMap<i32, ResponseFuture>>>,
}

impl QvOpenApiClientEventHandleable for FutureQvOpenApiClient {
    fn get_handler(&self) -> Arc<QvOpenApiClientMessageHandler> {
        self.delegate.get_handler().clone()
    }
}

impl FutureQvOpenApiClient {
    pub fn new() -> FutureQvOpenApiClient {
        Self::from(Box::new(QvOpenApiClient::new()))
    }

    pub fn from(client: Box<dyn AbstractQvOpenApiClient>) -> FutureQvOpenApiClient {
        let mut ret = FutureQvOpenApiClient {
            delegate: client,
            future_map_lock: Arc::new(Mutex::new(HashMap::new())),
        };

        {
            let lock = ret.future_map_lock.clone();
            ret.delegate.on_connect(Box::new(move |res| {
                Self::resolve(lock.clone(), CONNECT_TR_INDEX, res)
            }));
        }

        const BALANCE_TR_INDEX: i32 = 3;
        ret.delegate.on_data(|res| {
            // TODO
            info!("tr_index: {}, block_name: {}", res.tr_index, res.block_name.as_str());
            if res.tr_index == BALANCE_TR_INDEX {
                match res.block_name.as_str() {
                    qvopenapi::BLOCK_NAME_C8201_OUT => {
                        let data = res.block_data.downcast_ref::<C8201Response>().unwrap();
                        info!("출금가능금액: {}", data.chgm_pos_amtz16)
                    }
                    qvopenapi::BLOCK_NAME_C8201_OUT1_VEC => {
                        let data_vec = res.block_data.downcast_ref::<Vec<C8201Response1>>().unwrap();
                        for data in data_vec.iter() {
                            info!("종목번호: {}, 종목명: {}, 보유수: {}", data.issue_codez6, data.issue_namez40, data.bal_qtyz16)
                        }
                    }
                    _ => {
                        error!("Unknown block name {}", res.block_name)
                    }
                }
            }
        });

        ret
    }

    fn resolve(future_map_lock: Arc<Mutex<HashMap<i32, ResponseFuture>>>, tr_index: i32, res: Arc<dyn Any + Send + Sync>) {
        let mut future_map = future_map_lock.lock().unwrap();
        let future = future_map.get(&tr_index).unwrap();
        future.resolve(Ok(res));
        future_map.remove(&tr_index);
    }

    pub async fn connect(
        &self,
        new_hwnd: isize,
        account_type: qvopenapi::AccountType,
        id: &str,
        password: &str,
        cert_password: &str,
    ) -> Result<Arc<ConnectResponse>, QvOpenApiError> {
        let future: ResponseFuture = ResponseFuture::new();
        {
            let mut future_map = self.future_map_lock.lock().unwrap();
            if future_map.contains_key(&CONNECT_TR_INDEX) {
                return Err(QvOpenApiError::TransactionPoolFullError);
            }

            future_map.insert(CONNECT_TR_INDEX, future.clone());
        }
        match self.delegate.connect(new_hwnd, account_type, id, password, cert_password) {
            Ok(_) => downcast_future(future).await,
            Err(e) => {
                let mut future_map = self.future_map_lock.lock().unwrap();
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
