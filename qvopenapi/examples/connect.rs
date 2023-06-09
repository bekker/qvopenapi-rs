use std::time::Duration;

use ::log::*;
use qvopenapi::{SimpleQvOpenApiClient, QvOpenApiError, WindowHelper, C8201Response, C8201Response1};

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

    let mut client = SimpleQvOpenApiClient::new();
    let mut window_helper = WindowHelper::new();

    let hwnd = window_helper.run(&client)?;
    client.on_connect(|res| {
        info!("Connected: account count {}", res.account_count);
    });
    client.connect(
        hwnd,
        qvopenapi::AccountType::NAMUH,
        id.as_str(),
        password.as_str(),
        cert_password.as_str(),
    )?;
    std::thread::sleep(Duration::from_millis(3000));

    const BALANCE_TR_INDEX: i32 = 3;
    client.on_data(|res| {
        info!("tr_index: {}, block_name: {}", res.tr_index, res.block_name.as_str());
        if res.tr_index == BALANCE_TR_INDEX {
            match res.block_name.as_str() {
                qvopenapi::BLOCK_NAME_C8201_OUT => {
                    let data = res.block_data.downcast_ref::<C8201Response>().unwrap();
                    info!("출금가능금액: {}", data.chgm_pos_amtz16)
                }
                qvopenapi::BLOCK_NAME_C8201_OUT1_VEC => {
                    let data_vec = res.block_data.downcast_ref::<Vec<C8201Response1>>().unwrap();
                    for data in data_vec.iter() {
                        info!("종목번호: {}, 종목명: {}, 보유수: {}", data.issue_codez6, data.issue_namez40, data.bal_qtyz16)
                    }
                }
                _ => {
                    error!("Unknown block name {}", res.block_name)
                }
            }
        }
    });
    client.get_balance(BALANCE_TR_INDEX, 1, '1')?;
    std::thread::sleep(Duration::from_millis(3000));
    Ok(())
}

fn find_env(key: &str) -> Result<String, qvopenapi::QvOpenApiError> {
    std::env::var(key).map_err(|_| QvOpenApiError::BadRequestError {
        message: format!("env {} not found", key),
    })
}
