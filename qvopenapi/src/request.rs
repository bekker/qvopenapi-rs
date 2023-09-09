use crate::*;

pub trait QvOpenApiRequest: Send + Sync {
    fn before_post(&self) -> Result<(), QvOpenApiError>;
    fn call_lib(&self, hwnd: isize) -> Result<(), QvOpenApiError>;
    fn get_tr_code(&self) -> &str;
    fn get_tr_index(&self) -> i32;
}

pub const TR_CODE_CONNECT: &str = "_connect";

pub struct ConnectRequest {
    pub account_type: AccountType,
    pub id: String,
    pub password: String,
    pub cert_password: String,
}

#[derive(strum_macros::Display, Clone, Copy)]
pub enum AccountType {
    QV,
    NAMUH,
}

impl QvOpenApiRequest for ConnectRequest {
    fn before_post(&self) -> Result<(), QvOpenApiError> {
        if wmca_lib::is_connected()? {
            return Err(QvOpenApiError::AlreadyConnectedError);
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

    fn get_tr_code(&self) -> &str {
        TR_CODE_CONNECT
    }

    fn get_tr_index(&self) -> i32 {
        1
    }
}

pub struct QueryRequest<T: ?Sized> {
    pub tr_index: i32,
    pub tr_code: &'static str,
    pub input: Box<T>,
    pub account_index: i32,
}

impl<T: Send + Sync> QvOpenApiRequest for QueryRequest<T> {
    fn before_post(&self) -> Result<(), QvOpenApiError> {
        wmca_lib::assert_connected()
    }

    fn call_lib(&self, hwnd: isize) -> Result<(), QvOpenApiError> {
        wmca_lib::query(
            hwnd,
            self.tr_index,
            self.tr_code,
            self.input.as_ref(),
            self.account_index,
        )
    }

    fn get_tr_code(&self) -> &str {
        self.tr_code
    }

    fn get_tr_index(&self) -> i32 {
        self.tr_index
    }
}
