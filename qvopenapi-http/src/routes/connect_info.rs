use std::{convert::Infallible, sync::Arc};

use qvopenapi_future::QvOpenApiFutureClient;
use warp::{filters::{method::get, BoxedFilter}, Filter, reply::{Reply, self}, http::StatusCode};

use crate::error;

pub fn filter_connect_info(client: Arc<QvOpenApiFutureClient>) -> BoxedFilter<(impl Reply,)> {
    let cloned = client.clone();
    let handler = move || connect_info(cloned.clone());
    get()
        .and(warp::path!("connect-info"))
        .and_then(handler)
        .boxed()
}

async fn connect_info(client: Arc<QvOpenApiFutureClient>) -> Result<impl Reply, Infallible> {
    let ret = client.get_connect_info();
    if ret.is_err() {
        return error::convert_error(ret.err().unwrap());
    }

    let result = ret.unwrap();

    Ok(reply::with_status(
        reply::json(&result),
        StatusCode::OK,
    ))
}
