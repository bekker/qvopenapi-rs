extern crate qvopenapi_sys;
#[macro_use]
extern crate lazy_static;
mod error;
mod window_manager;

pub use error::*;

use log::*;
use once_cell::sync::OnceCell;
use qvopenapi_sys::WmcaLib;
use std::{ffi::CString, os::raw::c_char, sync::RwLock};
use window_manager::WindowManager;
use windows::Win32::{
    Foundation::{HWND, LPARAM, WPARAM},
    UI::WindowsAndMessaging::{SendMessageA, PostMessageA},
};

// Static mutables need wrappers like OnceCell or RwLock to prevent concurrency problem
static WMCA_LIB_CELL: OnceCell<WmcaLib> = OnceCell::new();
static WINDOW_MANAGER_LOCK: RwLock<WindowManager> = RwLock::new(WindowManager::new());

#[derive(strum_macros::Display)]
pub enum AccountType {
    QV,
    NAMUH,
}

/**
 * DLL을 로드하고 이벤트를 자동으로 처리할 윈도우를 생성한다.
 */
pub fn init() -> Result<(), QvOpenApiError> {
    bind_lib()?;
    window_manager::run_window(&WINDOW_MANAGER_LOCK)?;
    Ok(())
}

pub fn is_connected() -> Result<bool, QvOpenApiError> {
    let ret = (get_lib()?.is_connected)();
    debug!("is_connected {}", ret);
    Ok(ret != 0)
}

pub fn send_message(msg: u32, wparam: usize, lparam: isize) -> Result<(), QvOpenApiError> {
    unsafe {
        SendMessageA(HWND(get_hwnd()?), msg, WPARAM(wparam), LPARAM(lparam));
    }
    return Ok(());
}

pub fn post_message(msg: u32, wparam: usize, lparam: isize) -> Result<(), QvOpenApiError> {
    unsafe {
        PostMessageA(HWND(get_hwnd()?), msg, WPARAM(wparam), LPARAM(lparam));
    }
    return Ok(());
}

pub fn set_server(
    server: &str,
) -> Result<(), QvOpenApiError> {
    let server_cstr = make_c_string(server);
    c_bool_to_result((get_lib()?.set_server)(server_cstr.as_ptr()))
}

pub fn connect(
    account_type: AccountType,
    id: &str,
    password: &str,
    cert_password: &str,
) -> Result<(), QvOpenApiError> {
    let hwnd = get_hwnd()?;
    let msg = window_manager::WM_WMCAEVENT;
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

pub fn query(tr_code: &str, input: &str, account_index: i32) -> Result<(), QvOpenApiError> {
    let hwnd = get_hwnd()?;
    let tr_id: i32 = 0;
    let tr_code_cstr = make_c_string(tr_code);
    let input_cstr = make_c_string(input);

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

fn bind_lib() -> Result<(), QvOpenApiError> {
    info!("Loading wmca.dll");
    WMCA_LIB_CELL.set(qvopenapi_sys::bind_lib()?).unwrap();
    info!("Loaded wmca.dll");
    Ok(())
}

fn get_lib() -> Result<&'static WmcaLib, QvOpenApiError> {
    match WMCA_LIB_CELL.get() {
        Some(ref wmca_lib) => Ok(wmca_lib),
        None => Err(QvOpenApiError::WmcaDllNotLoadedError),
    }
}

fn get_hwnd() -> Result<isize, QvOpenApiError> {
    match WINDOW_MANAGER_LOCK.read().unwrap().hwnd {
        Some(hwnd) => Ok(hwnd),
        None => Err(QvOpenApiError::WindowNotCreatedError),
    }
}
