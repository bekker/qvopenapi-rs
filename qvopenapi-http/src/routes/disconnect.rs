use std::{convert::Infallible, sync::Arc};

use qvopenapi_async::QvOpenApiAsyncClient;
use warp::{
    filters::{method::post, BoxedFilter},
    http::StatusCode,
    reply::{self, Reply},
    Filter,
};

use crate::{error, response::HttpMessageResponse};

pub fn filter_disconnect(client: Arc<QvOpenApiAsyncClient>) -> BoxedFilter<(impl Reply,)> {
    let cloned = client.clone();
    let handler = move || disconnect(cloned.clone());
    post()
        .and(warp::path!("disconnect"))
        .and_then(handler)
        .boxed()
}

async fn disconnect(client: Arc<QvOpenApiAsyncClient>) -> Result<impl Reply, Infallible> {
    let ret = client.disconnect();
    if ret.is_err() {
        return error::convert_error(ret.err().unwrap());
    }

    Ok(reply::with_status(
        reply::json(&HttpMessageResponse {
            message: "OK".into(),
        }),
        StatusCode::OK,
    ))
}
