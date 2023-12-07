mod c8201;
pub use c8201::*;
use qvopenapi_bindings::OutDataBlock;
use serde::Serialize;

use std::ffi::c_char;
use serde_json::Value;

use crate::{error::*, utils::from_cp949_ptr, wmca_lib, client::QvOpenApiRequest};

pub fn parse_data(lparam: isize) -> std::result::Result<DataResponse, QvOpenApiError> {
    let data_block = lparam as *const OutDataBlock<c_char>;
    unsafe {
        let tr_index = (*data_block).tr_index;
        let data = (*data_block).p_data;
        let block_name = from_cp949_ptr((*data).block_name);
        let block_data = (*data).sz_data;
        let block_len = (*data).len;

        Ok(DataResponse {
            tr_index,
            block_data: parse_block(block_name.as_str(), block_data, block_len)?,
            block_name,
            block_len,
        })
    }
}

pub fn parse_sise(lparam: isize) -> std::result::Result<DataResponse, QvOpenApiError> {
    let data_block = lparam as *const OutDataBlock<c_char>;
    unsafe {
        let tr_index = (*data_block).tr_index;
        let data = (*data_block).p_data;
        let block_name = from_cp949_ptr((*data).block_name);
        let block_data = (*data).sz_data.offset(3); //앞쪽 3바이트는 패킷유형과 압축구분이므로 skip
        let block_len = (*data).len;

        Ok(DataResponse {
            tr_index,
            block_data: parse_block(block_name.as_str(), block_data, block_len)?,
            block_name,
            block_len,
        })
    }
}

fn parse_block(block_name: &str, block_data: *const c_char, block_len: i32) -> Result<Value, QvOpenApiError> {
    match block_name {
        BLOCK_NAME_C8201_OUT => parse_c8201_response(block_data, block_len),
        BLOCK_NAME_C8201_OUT1_ARRAY => parse_c8201_response1_array(block_data, block_len),
        _ => {
            Err(QvOpenApiError::UnimplementedBlockError { block_name: block_name.into() })
        }
    }
}

pub struct RawQueryRequest<T: ?Sized> {
    pub tr_code: &'static str,
    pub account_index: i32,
    pub raw_input: Box<T>,
}

impl<T: Send + Sync> QvOpenApiRequest for RawQueryRequest<T> {
    fn before_post(&self) -> Result<(), QvOpenApiError> {
        wmca_lib::assert_connected()
    }

    fn call_lib(&self, tr_index: i32, hwnd: isize) -> Result<(), QvOpenApiError> {
        wmca_lib::query(
            hwnd,
            tr_index,
            self.tr_code,
            self.raw_input.as_ref(),
            self.account_index,
        )
    }

    fn get_tr_code(&self) -> &str {
        self.tr_code
    }
}

impl <T> RawQueryRequest<T> {
    pub fn new(tr_code: &'static str, account_index: i32, raw_input: Box<T>) -> RawQueryRequest<T> {
        RawQueryRequest { tr_code, account_index, raw_input }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct DataResponse {
    pub tr_index: i32,
    pub block_name: String,
    pub block_len: i32,
    pub block_data: Value,
}

pub struct DisconnectRequest {
}

impl QvOpenApiRequest for DisconnectRequest {
    fn before_post(&self) -> Result<(), QvOpenApiError> {
        Ok(())
    }

    fn call_lib(&self, _tr_index: i32, _hwnd: isize) -> Result<(), QvOpenApiError> {
        wmca_lib::disconnect()
    }

    fn get_tr_code(&self) -> &str {
        "DISCONNECT"
    }
}
