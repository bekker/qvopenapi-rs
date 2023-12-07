use std::sync::Arc;

use qvopenapi_future::QvOpenApiFutureClient;
use warp::{filters::BoxedFilter, reply::Reply, Filter};

pub mod connect;
pub mod query;
pub mod connect_info;
pub mod disconnect;

pub fn filter(client: Arc<QvOpenApiFutureClient>) -> BoxedFilter<(impl Reply,)> {
    connect::filter_connect(client.clone())
        .or(query::filter_c8201(client.clone()))
        .or(connect_info::filter_connect_info(client.clone()))
        .or(disconnect::filter_disconnect(client.clone()))
        .boxed()
}
