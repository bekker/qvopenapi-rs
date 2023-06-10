extern crate qvopenapi_sys;
#[macro_use]
extern crate lazy_static;
mod basic_structs;
mod error;
mod message;
mod query;
mod request;
mod response;
mod window_mgr;
mod wmca_lib;
mod client;
mod bindings;

use std::{
    sync::{Arc},
};

pub use error::*;
use log::{debug, info};
pub use query::*;
pub use request::*;
pub use response::*;
pub use window_mgr::WindowHelper;
pub use wmca_lib::{init, is_connected, set_port, set_server};
pub use client::{QvOpenApiClient, QvOpenApiClientMessageHandler, SimpleQvOpenApiClient};

use windows::Win32::{
    Foundation::{HWND, LPARAM, WPARAM},
    UI::WindowsAndMessaging::{PostMessageA, WM_USER},
};

pub const WM_WMCAEVENT: u32 = WM_USER + 8400;

const CA_CUSTOM_EXECUTE_POSTED_COMMAND: u32 = WM_USER + 8410;
const CA_CONNECTED: u32 = WM_USER + 110;
const CA_DISCONNECTED: u32 = WM_USER + 120;
const CA_SOCKETERROR: u32 = WM_USER + 130;
const CA_RECEIVEDATA: u32 = WM_USER + 210;
const CA_RECEIVESISE: u32 = WM_USER + 220;
const CA_RECEIVEMESSAGE: u32 = WM_USER + 230;
const CA_RECEIVECOMPLETE: u32 = WM_USER + 240;
const CA_RECEIVEERROR: u32 = WM_USER + 250;

#[derive(strum_macros::Display, Clone, Copy)]
pub enum AccountType {
    QV,
    NAMUH,
}
