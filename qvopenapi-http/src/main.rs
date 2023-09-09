use std::convert::Infallible;

use ::log::*;
use serde::{Deserialize, Serialize};
use warp::{
    http::{StatusCode},
    *,
};

extern crate qvopenapi;
extern crate serde;
use qvopenapi::{AccountType, QvOpenApiError};

#[derive(Deserialize, Serialize)]
struct MessageResponse {
    message: String,
}

#[derive(Deserialize, Serialize)]
struct ConnectRequest {
    id: String,
    password: String,
    cert_password: String,
}

async fn do_run() -> Result<(), qvopenapi::QvOpenApiError> {
    env_logger::init_from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "debug"),
    );
    qvopenapi::init()?;

    let connect_filter = post()
        .and(path("connect"))
        .and(body::json())
        .and_then(connect);

    serve(connect_filter).run(([127, 0, 0, 1], 18000)).await;

    Ok(())
}

async fn connect(request: ConnectRequest) -> Result<impl Reply, Infallible> {
    let ret = qvopenapi::connect(
        AccountType::NAMUH,
        request.id.as_str(),
        request.password.as_str(),
        request.cert_password.as_str(),
    )
    .await;

    if ret.is_err() {
        return convert_error(ret.err().unwrap());
    }

    let connect_dto = ret.unwrap();

    Ok(reply::with_status(
        reply::json(&ConnectResponse {
            login_datetime: connect_dto.login_datetime.to_rfc3339(),
            server_name: connect_dto.server_name.clone(),
            user_id: connect_dto.user_id.clone(),
            account_count: connect_dto.account_count,
            account_infoes: connect_dto
                .account_infoes
                .iter()
                .map(|acc_info_dto| AccountInfoResponse {
                    account_no: acc_info_dto.account_no.clone(),
                    account_name: acc_info_dto.account_name.clone(),
                    act_pdt_cdz3: acc_info_dto.act_pdt_cdz3.clone(),
                    amn_tab_cdz4: acc_info_dto.amn_tab_cdz4.clone(),
                    expr_datez8: acc_info_dto.expr_datez8.clone(),
                    bulk_granted: acc_info_dto.bulk_granted,
                })
                .collect(),
        }),
        StatusCode::OK,
    ))
}

fn convert_error(err: QvOpenApiError) -> Result<reply::WithStatus<reply::Json>, Infallible> {
    match err {
        QvOpenApiError::AlreadyConnectedError => Ok(reply::with_status(
            reply::json(&MessageResponse {
                message: String::from(QvOpenApiError::AlreadyConnectedError.to_string()),
            }),
            StatusCode::BAD_REQUEST,
        )),
        err => Ok(reply::with_status(
            reply::json(&MessageResponse {
                message: String::from(err.to_string()),
            }),
            StatusCode::INTERNAL_SERVER_ERROR,
        )),
    }
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
