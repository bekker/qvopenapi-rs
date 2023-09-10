use std::sync::Arc;

use qvopenapi::QvOpenApiClient;
use warp::{filters::BoxedFilter, reply::Reply, Filter};

pub mod connect;
pub mod query;

pub fn filter(hwnd: isize, client: Arc<QvOpenApiClient>) -> BoxedFilter<(impl Reply,)> {
    connect::filter_connect(hwnd, client.clone())
        .or(query::filter_c8201(client.clone()))
        .boxed()
}
