mod c8201;
pub use c8201::*;
use qvopenapi_bindings::OutDataBlock;

use std::ffi::c_char;
use serde_json::Value;

use crate::{QvOpenApiError, utils::from_cp949_ptr};

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
        BLOCK_NAME_C8201_OUT1_VEC => parse_c8201_response1(block_data, block_len),
        _ => {
            Err(QvOpenApiError::UnimplementedBlockError { block_name: block_name.into() })
        }
    }
}

pub struct DataResponse {
    pub tr_index: i32,
    pub block_name: String,
    pub block_len: i32,
    pub block_data: Value,
}
