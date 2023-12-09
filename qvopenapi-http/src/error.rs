use std::convert::Infallible;

use warp::{
    http::StatusCode,
    *,
};
use qvopenapi_async::error::*;

use crate::response::HttpMessageResponse;

pub fn convert_error(err: QvOpenApiError) -> Result<reply::WithStatus<reply::Json>, Infallible> {
    match err {
        QvOpenApiError::AlreadyConnectedError => Ok(reply::with_status(
            reply::json(&HttpMessageResponse {
                message: err.to_string(),
            }),
            StatusCode::BAD_REQUEST,
        )),
        err => Ok(reply::with_status(
            reply::json(&HttpMessageResponse {
                message: err.to_string(),
            }),
            StatusCode::INTERNAL_SERVER_ERROR,
        )),
    }
}
