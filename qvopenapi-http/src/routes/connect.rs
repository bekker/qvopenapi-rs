use std::{convert::Infallible, sync::Arc};

use log::debug;
use qvopenapi_future::{QvOpenApiFutureClient, models::ConnectRequest};
use warp::{filters::{method::post, body, BoxedFilter}, Filter, reply::{Reply, self}, http::StatusCode};

use crate::error;

pub fn filter_connect(client: Arc<QvOpenApiFutureClient>) -> BoxedFilter<(impl Reply,)> {
    let cloned = client.clone();
    let handler = move |req: ConnectRequest| connect(cloned.clone(), req);
    post()
        .and(warp::path!("connect"))
        .and(body::json())
        .and_then(handler)
        .boxed()
}

async fn connect(client: Arc<QvOpenApiFutureClient>, request: ConnectRequest) -> Result<impl Reply, Infallible> {
    debug!("Connecting...");
    let ret = client.connect(request.account_type, &request.id, &request.password, &request.cert_password).await;
    debug!("Connect request sent");
    if ret.is_err() {
        return error::convert_error(ret.err().unwrap());
    }

    let result = ret.unwrap();

    let error = result.get("error");
    if error.is_some() && !error.unwrap().is_null() {
        return Ok(reply::with_status(
            reply::json(&result),
            StatusCode::INTERNAL_SERVER_ERROR,
        ))
    }

    Ok(reply::with_status(
        reply::json(&result),
        StatusCode::OK,
    ))
}
