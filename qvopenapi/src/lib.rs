extern crate qvopenapi_sys;
#[macro_use]
extern crate lazy_static;
mod error;
mod window_manager;

pub use error::*;

use qvopenapi_sys::WmcaLib;
use once_cell::sync::OnceCell;
use std::sync::RwLock;
use window_manager::WindowManager;

// Static mutables need wrappers like OnceCell or RwLock to prevent concurrency problem
static WMCA_LIB_CELL: OnceCell<WmcaLib> = OnceCell::new();
static WINDOW_MANAGER_LOCK: RwLock<WindowManager> = RwLock::new(WindowManager::new());

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

fn bind_lib() -> Result<(), QvOpenApiError> {
    println!("Loading wmca.dll");
    WMCA_LIB_CELL.set(qvopenapi_sys::bind_lib()?).unwrap();
    println!("Loaded wmca.dll");
    Ok(())
}

fn get_lib() -> Result<&'static WmcaLib, QvOpenApiError> {
    match WMCA_LIB_CELL.get() {
        Some(ref wmca_lib) => Ok(wmca_lib),
        None => Err(QvOpenApiError::WmcaDllNotLoadedError),
    }
}
