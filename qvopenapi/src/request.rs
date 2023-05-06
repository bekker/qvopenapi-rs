use std::{
    collections::VecDeque,
    sync::{Arc, RwLock},
};

use crate::*;

lazy_static! {
    static ref REQUEST_QUEUE_LOCK: RwLock<VecDeque<Arc<dyn WmcaRequest + Send + Sync>>> =
        RwLock::new(VecDeque::new());
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

pub fn post_request(cmd: Arc<dyn WmcaRequest + Send + Sync>) -> Result<(), QvOpenApiError> {
    let mut cmd_queue = REQUEST_QUEUE_LOCK.write().unwrap();
    cmd_queue.push_back(cmd);
    window_mgr::post_message(
        message::WM_CUSTOMEVENT,
        message::CA_COMMAND.try_into().unwrap(),
        0,
    )
}

pub fn execute_command() -> Result<(), QvOpenApiError> {
    let mut cmd_queue = REQUEST_QUEUE_LOCK.write().unwrap();
    let cmd = cmd_queue.pop_front();

    if cmd.is_none() {
        return Ok(());
    }

    cmd.unwrap().call_lib()
}
