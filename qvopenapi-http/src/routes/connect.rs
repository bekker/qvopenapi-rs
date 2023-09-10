use std::{convert::Infallible, sync::Arc};

use qvopenapi::{ConnectRequest, QvOpenApiClient, AbstractQvOpenApiClient};
use warp::{filters::{method::post, body, BoxedFilter}, Filter, reply::{Reply, self}, http::StatusCode};

use crate::{response::HttpMessageResponse, error};

pub fn filter_connect(hwnd: isize, client: Arc<QvOpenApiClient>) -> BoxedFilter<(impl Reply,)> {
    let cloned = client.clone();
    let handler = move |req: ConnectRequest| connect(hwnd, cloned.clone(), req);
    post()
        .and(warp::path!("connect"))
        .and(body::json())
        .and_then(handler)
        .boxed()
}

async fn connect(hwnd: isize, client: Arc<QvOpenApiClient>, request: ConnectRequest) -> Result<impl Reply, Infallible> {
    let ret = client.connect(hwnd, request.account_type, &request.id, &request.password, &request.cert_password);

    if ret.is_err() {
        return error::convert_error(ret.err().unwrap());
    }

    Ok(reply::with_status(
        reply::json(&HttpMessageResponse {
            message: "OK".into()
        }),
        StatusCode::OK,
    ))
}
