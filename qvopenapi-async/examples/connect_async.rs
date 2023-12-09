use std::io::Write;

use ::log::*;
use qvopenapi_async::{*, models::*, error::*};
use rpassword::read_password;

fn main() {
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

async fn do_run() -> Result<(), QvOpenApiError> {
    env_logger::init_from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "debug"),
    );

    let id = find_env_or_get_input("QV_ID")?;
    let password = find_env_or_get_input("QV_PW")?;
    let cert_password = find_env_or_get_input("QV_CERTPW")?;

    // Create future client
    let future_client = QvOpenApiAsyncClient::new()?;

    // Connect and query C8201 (계좌 잔고)
    let connect_response = future_client.connect(
        AccountType::NAMUH,
        id.as_str(),
        password.as_str(),
        cert_password.as_str(),
    ).await?;
    info!("connect response: {}", connect_response);

    let query_response = future_client.query(C8201Request::new( 1, '1').into_raw()).await?;
    info!("query response: {}", query_response);

    Ok(())
}

fn find_env_or_get_input(key: &str) -> Result<String, QvOpenApiError> {
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
