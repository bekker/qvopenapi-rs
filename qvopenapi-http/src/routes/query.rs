use std::{convert::Infallible, sync::Arc};

use qvopenapi::{QvOpenApiClient, AbstractQvOpenApiClient, C8201Request};
use warp::{filters::{method::post, body, BoxedFilter}, Filter, reply::{Reply, self}, http::StatusCode};

use crate::{response::HttpMessageResponse, error};

pub fn filter_c8201(client: Arc<QvOpenApiClient>) -> BoxedFilter<(impl Reply,)> {
    let cloned = client.clone();
    let handler = move |req: C8201Request| query_c8201(cloned.clone(), req);
    post()
        .and(warp::path!("query" / "c8201"))
        .and(body::json())
        .and_then(handler)
        .boxed()
}

async fn query_c8201(client: Arc<QvOpenApiClient>, request: C8201Request) -> Result<impl Reply, Infallible> {
    let ret = client.query(request.into_raw());

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
