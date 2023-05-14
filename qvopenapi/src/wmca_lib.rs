use crate::error::*;
use crate::*;

use log::*;
use once_cell::sync::OnceCell;
use qvopenapi_sys::WmcaLib;
use std::{ffi::CString, os::raw::c_char};

// Static mutables need wrappers like OnceCell to prevent concurrency problem
static WMCA_LIB_CELL: OnceCell<WmcaLib> = OnceCell::new();

/**
 * DLL을 미리 로드
 */
pub fn init() -> Result<(), QvOpenApiError> {
    get_lib()?;
    Ok(())
}

pub fn assert_connected() -> Result<(), QvOpenApiError> {
    match is_connected()? {
        true => Ok(()),
        false => Err(QvOpenApiError::NotConnectedError)
    }
}

pub fn is_connected() -> Result<bool, QvOpenApiError> {
    let ret = (get_lib()?.is_connected)();
    Ok(ret != 0)
}

pub fn set_server(server: &str) -> Result<(), QvOpenApiError> {
    let server_cstr = make_c_string(server);
    c_bool_to_result((get_lib()?.set_server)(server_cstr.as_ptr()))
}

pub fn set_port(port: i32) -> Result<(), QvOpenApiError> {
    c_bool_to_result((get_lib()?.set_port)(port))
}

pub fn connect(
    hwnd: isize,
    account_type: AccountType,
    id: &str,
    password: &str,
    cert_password: &str,
) -> Result<(), QvOpenApiError> {
    let msg = crate::WM_WMCAEVENT;
    let media_type = match account_type {
        AccountType::QV => 'P',
        AccountType::NAMUH => 'T',
    } as c_char;
    let user_type = match account_type {
        AccountType::QV => '1',
        AccountType::NAMUH => 'W',
    } as c_char;

    debug!(
        "connect ({}, {}, {}, {}, \"{}\", **, **)",
        hwnd, msg, media_type, user_type, id
    );

    let id_cstr = make_c_string(id);
    let password_cstr = make_c_string(password);
    let cert_password_cstr = make_c_string(cert_password);

    c_bool_to_result((get_lib()?.connect)(
        hwnd,
        msg,
        media_type,
        user_type,
        id_cstr.as_ptr(),
        password_cstr.as_ptr(),
        cert_password_cstr.as_ptr(),
    ))
}

pub fn query(hwnd: isize, tr_code: &str, input: &str, account_index: i32) -> Result<(), QvOpenApiError> {
    let tr_id: i32 = 0;
    let tr_code_cstr = make_c_string(tr_code);
    let input_cstr = make_c_string(input);

    debug!("query ({})", tr_code);

    c_bool_to_result((get_lib()?.query)(
        hwnd,
        tr_id,
        tr_code_cstr.as_ptr(),
        input_cstr.as_ptr(),
        input.len() as i32,
        account_index,
    ))
}

fn c_bool_to_result(val: i32) -> Result<(), QvOpenApiError> {
    debug!("c_bool_to_result {}", val);
    match val {
        // FIXME
        //0 => Err(QvOpenApiError::ReturnCodeError{code: val}),
        _ => Ok(()),
    }
}

fn make_c_string(original: &str) -> CString {
    CString::new(original).unwrap()
}

fn get_lib() -> Result<&'static WmcaLib, QvOpenApiError> {
    WMCA_LIB_CELL.get_or_try_init(bind_lib)
}

fn bind_lib() -> Result<WmcaLib, QvOpenApiError> {
    info!("Loading wmca.dll");
    let lib = qvopenapi_sys::bind_lib()?;
    info!("Loaded wmca.dll");
    Ok(lib)
}
