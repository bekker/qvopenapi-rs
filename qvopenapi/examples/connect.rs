use std::time::Duration;

use ::log::*;
use qvopenapi::{QvOpenApiClient, QvOpenApiError, WindowHelper};

fn main() {
    match do_run() {
        Ok(_) => {}
        Err(e) => {
            error!("Error occured: {}", e);
        }
    }
}

fn do_run() -> Result<(), qvopenapi::QvOpenApiError> {
    env_logger::init_from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "debug"),
    );

    let id = find_env("QV_ID")?;
    let password = find_env("QV_PW")?;
    let cert_password = find_env("QV_CERTPW")?;

    qvopenapi::init()?;

    let mut client = QvOpenApiClient::new();
    client.on_connect(|res| {
        info!("Connected: account count {}", res.account_count);
    });
    let mut window_helper = WindowHelper::new();

    let hwnd = window_helper.run(&client)?;
    client.connect(
        hwnd,
        qvopenapi::AccountType::NAMUH,
        id,
        password,
        cert_password,
    )?;
    std::thread::sleep(Duration::from_millis(3000));

    Ok(())
}

fn find_env(key: &str) -> Result<String, qvopenapi::QvOpenApiError> {
    std::env::var(key).map_err(|_| QvOpenApiError::BadRequestError {
        message: format!("env {} not found", key),
    })
}
