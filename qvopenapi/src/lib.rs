extern crate qvopenapi_sys;
#[macro_use]
extern crate lazy_static;
mod error;
mod message;
mod request;
mod window_mgr;
mod wmca_lib;

use std::sync::Arc;

pub use error::*;
pub use wmca_lib::{init, is_connected};

#[derive(strum_macros::Display, Clone, Copy)]
pub enum AccountType {
    QV,
    NAMUH,
}

pub fn connect(
    account_type: AccountType,
    id: &str,
    password: &str,
    cert_password: &str,
) -> Result<(), QvOpenApiError> {
    let cmd = request::ConnectRequest {
        account_type: account_type,
        id: id.to_string(),
        password: password.to_string(),
        cert_password: cert_password.to_string(),
    };
    request::post_request(Arc::new(cmd))
}

pub fn query(tr_code: &str, input: &str, account_index: i32) -> Result<(), QvOpenApiError> {
    let cmd = request::QueryRequest {
        tr_code: tr_code.to_string(),
        input: input.to_string(),
        account_index: account_index,
    };
    request::post_request(Arc::new(cmd))
}
