use crate::*;

use wmca_lib;

pub trait QvOpenApiRequest {
    fn before_post(&self) -> Result<(), QvOpenApiError>;
    fn call_lib(&self, hwnd: isize) -> Result<(), QvOpenApiError>;
}

pub struct ConnectRequest {
    pub account_type: AccountType,
    pub id: String,
    pub password: String,
    pub cert_password: String,
}

impl QvOpenApiRequest for ConnectRequest {
    fn before_post(&self) -> Result<(), QvOpenApiError> {
        if wmca_lib::is_connected()? {
            return Err(QvOpenApiError::AlreadyConnectedError)
        }

        Ok(())
    }

    fn call_lib(&self, hwnd: isize) -> Result<(), QvOpenApiError> {
        wmca_lib::connect(
            hwnd,
            self.account_type,
            &self.id,
            &self.password,
            &self.cert_password,
        )
    }
}

pub struct QueryRequest {
    pub tr_code: String,
    pub input: String,
    pub account_index: i32,
}

impl QvOpenApiRequest for QueryRequest {
    fn before_post(&self) -> Result<(), QvOpenApiError> {
        wmca_lib::assert_connected()
    }

    fn call_lib(&self, hwnd: isize) -> Result<(), QvOpenApiError> {
        wmca_lib::query(hwnd, &self.tr_code, &self.input, self.account_index)
    }
}
