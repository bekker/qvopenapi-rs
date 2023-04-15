use qvopenapi_sys::WmcaLib;

extern crate qvopenapi_sys;
#[macro_use]
extern crate lazy_static;
mod error;
mod window;

pub use error::*;
use once_cell::sync::OnceCell;
use std::sync::RwLock;
use window::{WindowManager, WindowManagerStatus};
use windows::Win32::Foundation::HWND;

static WMCA_LIB: OnceCell<WmcaLib> = OnceCell::new();
static WINDOW_MANAGER: RwLock<WindowManager> = RwLock::new(WindowManager::new());

pub fn init() -> Result<(), QvOpenApiError> {
    load_lib()?;
    window::run_window(&WINDOW_MANAGER)?;
    Ok(())
}

fn load_lib() -> Result<(), QvOpenApiError> {
    println!("Loading wmca.dll");
    WMCA_LIB.set(qvopenapi_sys::load_lib()?).unwrap();
    println!("Loaded wmca.dll");
    Ok(())
}

fn get_lib() -> Result<&'static WmcaLib, QvOpenApiError> {
    match WMCA_LIB.get() {
        Some(ref wmca_lib) => Ok(wmca_lib),
        None => Err(QvOpenApiError::WmcaDllNotLoadedError),
    }
}

pub fn is_connected() -> Result<bool, QvOpenApiError> {
    Ok((get_lib()?.is_connected)() != 0)
}
