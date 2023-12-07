use std::{collections::HashMap, sync::{Arc, Mutex, RwLock}};

use crate::context::*;
use log::{info, error};
use qvopenapi::{AbstractQvOpenApiClient, error::*, models::*, QvOpenApiRequest, WindowHelper, QvOpenApiClient};

type TrContextMap = HashMap<i32, Arc<TrContext>>;
const INITIAL_TR_INDEX: i32 = 3;
const MAX_TR_INDEX: i32 = 255;

pub struct QvOpenApiFutureClient {
    delegate: Arc<dyn AbstractQvOpenApiClient + Send + Sync>,
    tr_context_map: Arc<RwLock<TrContextMap>>,
    next_tr_index: Mutex<i32>,
    is_connecting: Arc<RwLock<bool>>,
    hwnd: isize,
}

impl QvOpenApiFutureClient {
    pub fn new() -> Result<QvOpenApiFutureClient, QvOpenApiError> {
        // Create a window
        let client = Arc::new(QvOpenApiClient::new()?);
        let window_helper = WindowHelper::new();
        let hwnd = window_helper.run(client.as_ref())?;
        Ok(Self::new_custom(client.clone(), hwnd))
    }

    pub fn new_custom(delgate: Arc<dyn AbstractQvOpenApiClient + Send + Sync>, hwnd: isize) -> QvOpenApiFutureClient {
        let client = QvOpenApiFutureClient {
            delegate: delgate.clone(),
            tr_context_map: Arc::new(RwLock::new(HashMap::new())),
            next_tr_index: Mutex::new(INITIAL_TR_INDEX),
            is_connecting: Arc::new(RwLock::new(false)),
            hwnd,
        };

        client.setup_callbacks(delgate);

        client
    }

    fn set_context(&self, tr_index: i32, tr_type: TrType) -> Result<Arc<TrContext>, QvOpenApiError> {
        let mut map = self.tr_context_map.write().unwrap();
        if map.contains_key(&tr_index) {
            Err(QvOpenApiError::BadRequestError { message: format!("Already using tr_index {}", tr_index) })
        } else {
            let context = Arc::new(TrContext::new(tr_type.clone()));
            map.insert(tr_index, context.clone());

            if matches!(tr_type, TrType::CONNECT) {
                let mut locked = self.is_connecting.write().unwrap();
                *locked = true;
            }

            Ok(context)
        }
    }

    pub fn connect(
        &self,
        account_type: AccountType,
        id: &str,
        password: &str,
        cert_password: &str,
    ) -> TrFuture {
        TrFuture::new(self.do_connect(self.hwnd, account_type, id, password, cert_password))
    }

    fn do_connect(
        &self,
        new_hwnd: isize,
        account_type: AccountType,
        id: &str,
        password: &str,
        cert_password: &str,
    ) -> Result<Arc<TrContext>, QvOpenApiError> {
        let context = self.set_context(TR_INDEX_CONNECT, TrType::CONNECT)?;
        self.delegate.connect(new_hwnd, account_type, id, password, cert_password)?;
        Ok(context)
    }

    pub fn query(
        &self,
        req: Arc<dyn QvOpenApiRequest>
    ) -> TrFuture {
        TrFuture::new(self.do_query(req))
    }

    fn do_query(
        &self,
        req: Arc<dyn QvOpenApiRequest>
    ) -> Result<Arc<TrContext>, QvOpenApiError> {
        let tr_index = self.get_next_tr_index();
        let context = self.set_context(tr_index, TrType::QUERY)?;
        self.delegate.query(tr_index, req)?;
        Ok(context)
    }

    fn setup_callbacks(&self, delagate: Arc<dyn AbstractQvOpenApiClient + Send + Sync>) {
        delagate.on_connect(Box::new(|res| {
            info!("Connected: account count {}", res.account_count);
        }));
        {
            let context_map_lock = self.tr_context_map.clone();
            let is_connecting_lock = self.is_connecting.clone();
            delagate.on_connect(Box::new(move |res| {
                let mut is_connecting_locked = is_connecting_lock.write().unwrap();
                Self::handle_callback(context_map_lock.clone(), TR_INDEX_CONNECT, res, |context, res| { context.on_connect(res) });
                *is_connecting_locked = false;
            }));
        }
        {
            let context_map_lock = self.tr_context_map.clone();
            delagate.on_data(Box::new(move |res| {
                Self::handle_callback(context_map_lock.clone(), res.tr_index, res, |context, res| { context.on_data(res) });
            }));
        }
        {
            let context_map_lock = self.tr_context_map.clone();
            delagate.on_complete(Box::new(move |tr_index| {
                Self::handle_callback(context_map_lock.clone(), tr_index, (), |context, _res| { context.on_complete() });
            }));
        }
        {
            let context_map_lock = self.tr_context_map.clone();
            let is_connecting_lock = self.is_connecting.clone();
            delagate.on_disconnect(Box::new(move || {
                let mut is_connecting_locked = is_connecting_lock.write().unwrap();
                // make all requests end when disconnect
                let context_map = context_map_lock.write().unwrap();
                for context in context_map.values() {
                    context.on_disconnect();
                }
                *is_connecting_locked = false;
            }));
        }
        {
            let context_map_lock = self.tr_context_map.clone();
            let is_connecting_lock = self.is_connecting.clone();
            delagate.on_message(Box::new(move |res| {
                let is_connecting_locked = is_connecting_lock.read().unwrap();

                // If connecting, all messages should direct to connect context
                let tr_index = match *is_connecting_locked {
                    true => TR_INDEX_CONNECT,
                    false => res.tr_index
                };
                Self::handle_callback(context_map_lock.clone(), tr_index, (), |context, _res| { context.on_message(res.clone()) })
            }));
        }
    }

    fn handle_callback<F, R>(context_map_lock: Arc<RwLock<TrContextMap>>, tr_index: i32, res: R, callback: F) where F: Fn(&TrContext, R) -> bool {
        let mut completed = false;
        {
            let context_map = context_map_lock.read().unwrap();
            match context_map.get(&tr_index) {
                Some(context) => {
                    completed = callback(context, res);
                },
                None => {
                    if tr_index != TR_INDEX_CONNECT {
                        error!("Context for tr_index {} is not found", tr_index);
                    }
                }
            }
        }

        if completed {
            let mut context_map = context_map_lock.write().unwrap();
            context_map.remove(&tr_index);
        }
    }

    fn get_next_tr_index(&self) -> i32 {
        let mut locked = self.next_tr_index.lock().unwrap();
        let ret: i32 = *locked;
        *locked += 1;
        if *locked > MAX_TR_INDEX {
            *locked = INITIAL_TR_INDEX;
        }
        ret
    }
}
