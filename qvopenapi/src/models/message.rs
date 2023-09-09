use crate::*;
use crate::utils::from_cp949;
use crate::utils::from_cp949_ptr;
use log::*;
use qvopenapi_bindings::MessageHeader;
use qvopenapi_bindings::OutDataBlock;
use std::ffi::c_char;

use crate::error::*;

pub fn parse_message(lparam: isize) -> std::result::Result<MessageResponse, QvOpenApiError> {
    let data_block = lparam as *const OutDataBlock<MessageHeader>;
    unsafe {
        let tr_index = (*data_block).tr_index;
        let msg_header = (*(*data_block).p_data).sz_data;
        let msg_code = from_cp949(&(*msg_header).message_code);
        let msg = from_cp949(&(*msg_header).message).trim().into();

        Ok(MessageResponse {
            tr_index,
            msg_code,
            msg,
        })
    }
}

pub fn parse_complete(lparam: isize) -> std::result::Result<i32, QvOpenApiError> {
    let data_block = lparam as *const OutDataBlock<()>;
    unsafe {
        let tr_index = (*data_block).tr_index;
        info!("CA_RECEIVECOMPLETE [TR{}]", tr_index);
        Ok(tr_index)
    }
}

pub fn parse_error(lparam: isize) -> std::result::Result<ErrorResponse, QvOpenApiError> {
    let data_block = lparam as *const OutDataBlock<c_char>;
    unsafe {
        let tr_index = (*data_block).tr_index;
        let error_msg = from_cp949_ptr((*(*data_block).p_data).sz_data).trim().into();
        Ok(ErrorResponse {
            tr_index,
            error_msg,
        })
    }
}

pub struct MessageResponse {
    pub tr_index: i32,
    pub msg_code: String,
    pub msg: String,
}

pub struct ErrorResponse {
    pub tr_index: i32,
    pub error_msg: String,
}
