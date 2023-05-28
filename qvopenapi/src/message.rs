use crate::*;
use chrono::{FixedOffset, TimeZone};
use log::*;
use std::ffi::{c_char, CStr};

use crate::basic_structs::*;
use crate::error::*;
use encoding::{all::WINDOWS_949, DecoderTrap, Encoding};

lazy_static! {
    static ref SEOUL_TZ: FixedOffset = FixedOffset::east_opt(9 * 3600).unwrap();
}

pub fn parse_connect(lparam: isize) -> std::result::Result<ConnectResponse, QvOpenApiError> {
    let data_block = lparam as *const LoginBlock;
    unsafe {
        let login_info = (*data_block).login_info;
        let login_datetime_str = from_cp949(&(*login_info).login_datetime); // "20230506203715"
        let server_name = String::from(from_cp949(&(*login_info).server_name).trim()); // "htsi194        "
        let user_id = from_cp949(&(*login_info).user_id);
        let account_count_str = from_cp949(&(*login_info).account_count); // "002"

        let account_count: usize = account_count_str.parse().unwrap();

        let account_infoes = (*login_info)
            .account_infoes
            .iter()
            .take(account_count)
            .map(|account_info_raw| {
                let account_no = from_cp949(&account_info_raw.account_no);
                let account_name = String::from(from_cp949(&account_info_raw.account_name).trim());
                let act_pdt_cdz3 = from_cp949(&account_info_raw.act_pdt_cdz3);
                let amn_tab_cdz4 = from_cp949(&account_info_raw.amn_tab_cdz4);
                let expr_datez8 = String::from(from_cp949(&account_info_raw.expr_datez8).trim());
                let bulk_granted = account_info_raw.granted == 'G' as i8;
                AccountInfoResponse {
                    account_no,
                    account_name,
                    act_pdt_cdz3,
                    amn_tab_cdz4,
                    expr_datez8,
                    bulk_granted,
                }
            })
            .collect();

        let login_datetime = SEOUL_TZ.datetime_from_str(&login_datetime_str, "%Y%m%d%H%M%S")?;

        Ok(ConnectResponse {
            login_datetime,
            server_name,
            user_id,
            account_count,
            account_infoes,
        })
    }
}

pub fn parse_message(lparam: isize) -> std::result::Result<MessageResponse, QvOpenApiError> {
    let data_block = lparam as *const OutDataBlock<MessageHeader>;
    unsafe {
        let tr_index = (*data_block).tr_index;
        let msg_header = (*(*data_block).p_data).sz_data;
        let msg_code = from_cp949(&(*msg_header).message_code);
        let msg = from_cp949(&(*msg_header).message);

        Ok(MessageResponse {
            tr_index,
            msg_code,
            msg,
        })
    }
}

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
            block_name,
            block_data,
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
            block_name,
            block_data,
            block_len,
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
        let error_msg = from_cp949_ptr((*(*data_block).p_data).sz_data);
        Ok(ErrorResponse {
            tr_index,
            error_msg,
        })
    }
}

/**
 * 문자열 끝에 null이 없을 수도, 있을 수도 있음
 */
pub fn from_cp949(src: &[c_char]) -> String {
    let mut cloned: Vec<u8> = vec![];
    for s in src.iter() {
        // null을 만나면 여기까지만
        if *s == 0 {
            break;
        }
        cloned.push(*s as u8);
    }
    WINDOWS_949
        .decode(cloned.as_slice(), DecoderTrap::Replace)
        .unwrap()
}

/**
 * 문자열 끝에 null이 있음
 */
pub fn from_cp949_ptr(src: *const c_char) -> String {
    unsafe {
        let cstr: &CStr = CStr::from_ptr(src);
        WINDOWS_949
            .decode(cstr.to_bytes(), DecoderTrap::Replace)
            .unwrap()
    }
}
