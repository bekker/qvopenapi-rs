use std::{sync::RwLock, collections::VecDeque};

use crate::*;

lazy_static! {
    static ref CURRENT_COMMAND_QUEUE_LOCK: RwLock<VecDeque<Box<dyn WmcaCommand + Send + Sync>>> = RwLock::new(VecDeque::new());
}

pub struct ConnectCommand {
    pub account_type: AccountType,
    pub id: String,
    pub password: String,
    pub cert_password: String,
}

impl WmcaCommand for ConnectCommand {
    fn execute(&self) -> Result<(), QvOpenApiError> {
        wmcalib::connect(self.account_type, &self.id, &self.password, &self.cert_password)
    }
}

pub struct QueryCommand {
    pub tr_code: String,
    pub input: String,
    pub account_index: i32,
}

impl WmcaCommand for QueryCommand {
    fn execute(&self) -> Result<(), QvOpenApiError> {
        wmcalib::query(&self.tr_code, &self.input, self.account_index)
    }
}

pub trait WmcaCommand {
    fn execute(&self) -> Result<(), QvOpenApiError>;
}

pub fn post_command(cmd: Box<dyn WmcaCommand + Send + Sync>) -> Result<(), QvOpenApiError> {
    let mut cmd_queue = CURRENT_COMMAND_QUEUE_LOCK.write().unwrap();
    cmd_queue.push_back(cmd);
    window_manager::post_message(
        window_manager::WM_CUSTOMEVENT,
        window_manager::CA_COMMAND.try_into().unwrap(),
        0)
}

pub fn execute_command() -> Result<(), QvOpenApiError> {
    let mut cmd_queue = CURRENT_COMMAND_QUEUE_LOCK.write().unwrap();
    let cmd = cmd_queue.pop_front();

    if cmd.is_none() {
        return Ok(())
    }

    cmd.unwrap().execute()
}
