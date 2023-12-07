use std::sync::Arc;

use ::log::*;
use warp::*;

extern crate qvopenapi_future;
extern crate serde;
mod error;
mod routes;
mod response;
use qvopenapi_future::{error::*, QvOpenApiFutureClient};

async fn do_run() -> Result<(), QvOpenApiError> {
    let client = set_up_client()?;

    serve(routes::filter(client))
        .run(([0, 0, 0, 0], 18000)).await;

    Ok(())
}

fn set_up_client() -> Result<Arc<QvOpenApiFutureClient>, QvOpenApiError> {
    let client = QvOpenApiFutureClient::new()?;
    Ok(Arc::new(client))
}

fn main() {
    env_logger::init_from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "debug"),
    );
    debug!("Starting up tokio runtime...");

    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_or_else(|e| {
            error!("Tokio runtime init error: {}", e.to_string())
        }, |rt| {
            debug!("Tokio runtime init complete");
            rt.block_on(async move {
                match do_run().await {
                    Ok(_) => {}
                    Err(e) => {
                        error!("Error occured: {}", e);
                    }
                }
            });
        });
}
