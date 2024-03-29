use std::{io::Write, time::Duration};

use ::log::*;
use qvopenapi::{error::*, models::*, AbstractQvOpenApiClient, QvOpenApiClient, WindowHelper};
use rpassword::read_password;

fn main() {
    match do_run() {
        Ok(_) => {}
        Err(e) => {
            error!("Error occured: {}", e);
        }
    }
}

fn do_run() -> Result<(), QvOpenApiError> {
    env_logger::init_from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "debug"),
    );

    let id = find_env_or_get_input("QV_ID")?;
    let password = find_env_or_get_input("QV_PW")?;
    let cert_password = find_env_or_get_input("QV_CERTPW")?;

    // Create a window
    let client = QvOpenApiClient::new()?;
    let window_helper = WindowHelper::new();
    let hwnd = window_helper.run(&client)?;

    // Setup callbacks
    const BALANCE_TR_INDEX: i32 = 3;
    client.on_connect(Box::new(|res| {
        info!("Connected: account count {}", res.account_count);
    }));
    client.on_data(Box::new(|res| {
        info!(
            "tr_index: {}, block_name: {}",
            res.tr_index,
            res.block_name.as_str()
        );
        if res.tr_index == BALANCE_TR_INDEX {
            match res.block_name.as_str() {
                BLOCK_NAME_C8201_OUT => {
                    let data = &res.block_data;
                    info!("출금가능금액: {}", data["chgm_pos_amtz16"])
                }
                BLOCK_NAME_C8201_OUT1_ARRAY => {
                    for data in res.block_data.as_array().unwrap().iter() {
                        info!(
                            "종목번호: {}, 종목명: {}, 보유수: {}",
                            data["issue_codez6"], data["issue_namez40"], data["bal_qtyz16"]
                        )
                    }
                }
                _ => {
                    error!("Unknown block name {}", res.block_name)
                }
            }
        }
    }));

    // Connect and query C8201 (계좌 잔고)
    client.connect(
        hwnd,
        AccountType::NAMUH,
        id.as_str(),
        password.as_str(),
        cert_password.as_str(),
    )?;
    std::thread::sleep(Duration::from_millis(3000));

    client.query(BALANCE_TR_INDEX, C8201Request::new(1, '1').into_raw())?;
    std::thread::sleep(Duration::from_millis(3000));
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
            });
        }

        let trimmed = String::from(input_result.unwrap().trim());
        if trimmed.len() > 0 {
            return Ok(trimmed);
        }

        println!("");
    }
}
