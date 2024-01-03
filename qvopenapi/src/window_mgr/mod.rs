use log::*;
use std::{
    sync::{Arc, RwLock},
    thread::JoinHandle,
};

use crate::{error::*, *};

pub mod message_const;

#[cfg(target_os = "windows")]
mod window_mgr_win32;
#[cfg(target_os = "windows")]
use window_mgr_win32::*;

#[cfg(not(target_os = "windows"))]
mod window_mgr_mock;
#[cfg(not(target_os = "windows"))]
use window_mgr_mock::*;

pub struct WindowHelper {
    pub hwnd: Option<isize>,
    pub status: WindowStatus,
    pub thread: Option<JoinHandle<std::result::Result<(), QvOpenApiError>>>,
}

#[derive(PartialEq, Eq)]
pub enum WindowStatus {
    Init,
    Created,
    Destroyed,
    Error,
}

impl Drop for WindowHelper {
    fn drop(&mut self) {
        self.destroy()
    }
}

impl WindowHelper {
    pub const fn new() -> Self {
        WindowHelper {
            hwnd: None,
            status: WindowStatus::Init,
            thread: None,
        }
    }

    pub fn run(
        &self,
        client: &dyn AbstractQvOpenApiClient,
    ) -> std::result::Result<isize, QvOpenApiError> {
        let ret = Arc::new(RwLock::new(WindowHelper {
            hwnd: None,
            status: WindowStatus::Init,
            thread: None,
        }));
        run_window_async(ret, client.get_handler())
    }

    pub fn destroy(&mut self) {
        if self.hwnd.is_some() && self.status != WindowStatus::Destroyed {
            info!("Destroying window...");
            unsafe {
                destroy_window(self.hwnd.unwrap());
            }
        }
        self.thread.take().map(JoinHandle::join);
    }
}

pub fn post_message_to_window(hwnd: isize, msg: u32, wparam: u32, lparam: isize) {
    unsafe {
        post_message(hwnd, msg, wparam, lparam);
    }
}
