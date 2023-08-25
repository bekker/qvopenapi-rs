use std::{time::Duration, io::Write};

use ::log::*;
use qvopenapi::{QvOpenApiError, WindowHelper};
use qvopenapi_future::FutureQvOpenApiClient;
use rpassword::read_password;

#[tokio::main]
async fn main() {
    match do_run().await {
        Ok(_) => {}
        Err(e) => {
            error!("Error occured: {}", e);
        }
    }
}

async fn do_run() -> Result<(), qvopenapi::QvOpenApiError> {
    env_logger::init_from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "debug"),
    );

    let id = find_env_or_get_input("QV_ID")?;
    let password = find_env_or_get_input("QV_PW")?;
    let cert_password = find_env_or_get_input("QV_CERTPW")?;

    qvopenapi::init()?;

    let client = FutureQvOpenApiClient::new();
    let mut window_helper = WindowHelper::new();

    let hwnd = window_helper.run(&client)?;
    let connect_res = client.connect(
        hwnd,
        qvopenapi::AccountType::NAMUH,
        id.as_str(),
        password.as_str(),
        cert_password.as_str(),
    ).await?;
    info!("Connected. user_id: {}, account count: {}, ", connect_res.user_id, connect_res.account_count);
    client.get_balance(3, 1, '1')?;
    std::thread::sleep(Duration::from_millis(3000));
    Ok(())
}

fn find_env_or_get_input(key: &str) -> Result<String, qvopenapi::QvOpenApiError> {
    let env_var = std::env::var(key);

    if env_var.is_ok() {
        return Ok(env_var.unwrap());
    }

    loop {
        print!("Enter {}: ", key);
        std::io::stdout().flush().unwrap();

        let input_result = read_password();
        if input_result.is_err() {
            return Err(QvOpenApiError::BadRequestError {
                message: format!("env {} not found", key),
            })
        }

        let trimmed = String::from(input_result.unwrap().trim());
        if trimmed.len() > 0 {
            return Ok(trimmed)
        }

        println!("");
    }
}
