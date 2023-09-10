use std::sync::Arc;

use callback::setup_callbacks;
use ::log::*;
use warp::*;

extern crate qvopenapi;
extern crate serde;
mod callback;
mod error;
mod routes;
mod response;
use qvopenapi::{QvOpenApiError, QvOpenApiClient, WindowHelper};

async fn do_run() -> Result<(), qvopenapi::QvOpenApiError> {
    env_logger::init_from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "debug"),
    );
    let (hwnd, client) = set_up_client()?;

    serve(routes::filter(hwnd, client))
        .run(([127, 0, 0, 1], 18000)).await;

    Ok(())
}

fn set_up_client() -> Result<(isize, Arc<QvOpenApiClient>), QvOpenApiError> {
    // Initialize DLL
    qvopenapi::init()?;

    // Create a window
    let mut client = QvOpenApiClient::new();
    let mut window_helper = WindowHelper::new();
    let hwnd = window_helper.run(&client)?;
    setup_callbacks(&mut client);

    Ok((hwnd, Arc::new(client)))
}

#[tokio::main]
async fn main() {
    match do_run().await {
        Ok(_) => {}
        Err(e) => {
            error!("Error occured: {}", e);
        }
    }
}
