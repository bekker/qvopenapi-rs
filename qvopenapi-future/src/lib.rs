extern crate qvopenapi;
#[macro_use]
extern crate lazy_static;
use std::sync::Arc;

pub mod client;
mod query;
mod request;

use qvopenapi::*;

pub fn connect(
    account_type: AccountType,
    id: &str,
    password: &str,
    cert_password: &str,
) -> ResponseFuture<ConnectResponse> {
    let req = ConnectRequest {
        account_type: account_type,
        id: id.to_string(),
        password: password.to_string(),
        cert_password: cert_password.to_string(),
    };
    request::post_request(Arc::new(req))
}

pub fn query(tr_code: &str, input: &str, account_index: i32) -> ResponseFuture<QueryResponse> {
    let req = QueryRequest {
        tr_code: tr_code.to_string(),
        input: input.to_string(),
        account_index: account_index,
    };
    request::post_request(Arc::new(req))
}
