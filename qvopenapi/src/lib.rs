extern crate qvopenapi_sys;
#[macro_use]
extern crate lazy_static;
mod error;
mod window_manager;

pub use error::*;

use qvopenapi_sys::WmcaLib;
use once_cell::sync::OnceCell;
use std::{sync::RwLock, os::raw::c_char, ffi::{CString}};
use window_manager::WindowManager;
use log::*;

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
    Ok((get_lib()?.is_connected)() != 0)
}

pub fn connect(account_type: AccountType, id: &str, password: &str, cert_password: &str) -> Result<(), QvOpenApiError> {
    let hwnd = get_hwnd()?;
    let msg = window_manager::CA_WMCAEVENT;
    let media_type = match account_type {
        AccountType::QV => 'P',
        AccountType::NAMUH => 'T',
    } as c_char;
    let user_type = match account_type {
        AccountType::QV => '1',
        AccountType::NAMUH => 'W',
    } as c_char;

    debug!("connect ({}, {}, {}, \"{}\", **, **)", hwnd, msg, account_type, id);

    c_bool_to_result((get_lib()?.connect)(
        hwnd,
        msg,
        media_type,
        user_type,
        empty_if_null(id).as_ptr(),
        empty_if_null(password).as_ptr(),
        empty_if_null(cert_password).as_ptr(),
    ))
}

fn c_bool_to_result(val: i32) -> Result<(), QvOpenApiError> {
    debug!("return {}", val);
    match val {
        0 => Err(QvOpenApiError::ReturnCodeError{code: val}),
        _ => Ok(()),
    }
}

fn empty_if_null(original: &str) -> CString {
    CString::new(original).unwrap_or(CString::new("").unwrap())
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
