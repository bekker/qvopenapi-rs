use std::{convert::Infallible, sync::Arc};

use qvopenapi_future::{models::*, QvOpenApiFutureClient};
use warp::{filters::{method::post, body, BoxedFilter}, Filter, reply::{Reply, self}, http::StatusCode};

use crate::error;

pub fn filter_c8201(client: Arc<QvOpenApiFutureClient>) -> BoxedFilter<(impl Reply,)> {
    let cloned = client.clone();
    let handler = move |req: C8201Request| query_c8201(cloned.clone(), req);
    post()
        .and(warp::path!("query" / "c8201"))
        .and(body::json())
        .and_then(handler)
        .boxed()
}

async fn query_c8201(client: Arc<QvOpenApiFutureClient>, request: C8201Request) -> Result<impl Reply, Infallible> {
    let ret = client.query( request.into_raw()).await;

    if ret.is_err() {
        return error::convert_error(ret.err().unwrap());
    }

    Ok(reply::with_status(
        reply::json(&ret.unwrap()),
        StatusCode::OK,
    ))
}
