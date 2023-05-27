extern crate qvopenapi_sys;
#[macro_use]
extern crate lazy_static;
mod basic_structs;
mod error;
mod message;
mod request;
mod response;
mod wmca_lib;
mod window_mgr;

use std::{collections::VecDeque, sync::Arc};

pub use error::*;
use log::debug;
pub use request::*;
pub use response::*;
pub use wmca_lib::{init, is_connected, set_server, set_port};
pub use window_mgr::WindowHelper;

use windows::Win32::{UI::WindowsAndMessaging::{WM_USER, PostMessageA}, Foundation::{HWND, WPARAM, LPARAM}};

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

pub struct QvOpenApiClient {
    hwnd: Option<isize>,
    pub on_connect: fn(&ConnectResponse),
    pub on_disconnect: fn(),
    pub on_socket_error: fn(),
    pub on_data: fn(),
    pub on_sise: fn(),
    pub on_message: fn(&MessageResponse),
    pub on_complete: fn(tr_index: i32),
    pub on_error: fn(error_msg: &str),
    request_queue: VecDeque<Arc<dyn QvOpenApiRequest>>,
}

impl QvOpenApiClient {
    pub fn new() -> QvOpenApiClient {
        QvOpenApiClient {
            hwnd: None,
            on_connect: |_| {},
            on_disconnect: || {},
            on_socket_error: || {},
            on_data: || {},
            on_sise: || {},
            on_message: |_| {},
            on_complete: |_| {},
            on_error: |_| {},
            request_queue: VecDeque::new(),
        }
    }

    pub fn connect(
        &mut self,
        hwnd: isize,
        req: ConnectRequest,
    ) -> Result<(), QvOpenApiError> {
        self.hwnd = Some(hwnd);
        self.post_command(Arc::new(req))
    }

    pub fn query(&mut self, req: Arc<QueryRequest>) -> Result<(), QvOpenApiError> {
        self.post_command(req)
    }

    fn post_command(&mut self, command: Arc<dyn QvOpenApiRequest>) -> Result<(), QvOpenApiError> {
        command.before_post()?;
        self.request_queue.push_back(command);
        post_message_to_window(self.hwnd.unwrap(), WM_WMCAEVENT, CA_CUSTOM_EXECUTE_POSTED_COMMAND, 0);
        Ok(())
    }
}

impl window_mgr::WmcaMessageHandleable for QvOpenApiClient {
    fn on_wmca_msg(&mut self, wparam: usize, lparam: isize) -> std::result::Result<(), QvOpenApiError> {
        debug!("on_wmca_msg {} {}", wparam, lparam);
        match u32::try_from(wparam).unwrap() {
            CA_CONNECTED => {
                let res = message::parse_connect(lparam)?;
                (self.on_connect)(&res);
                Ok(())
            },
            CA_DISCONNECTED => {
                (self.on_disconnect)();
                Ok(())
            },
            CA_SOCKETERROR => {
                (self.on_socket_error)();
                Ok(())
            },
            CA_RECEIVEDATA => {
                (self.on_data)();
                Ok(())
            },
            CA_RECEIVESISE => {
                (self.on_sise)();
                Ok(())
            },
            CA_RECEIVEMESSAGE => {
                let res = message::parse_message(lparam)?;
                (self.on_message)(&res);
                Ok(())
            },
            CA_RECEIVECOMPLETE => {
                let res = message::parse_complete(lparam)?;
                (self.on_complete)(res);
                Ok(())
            },
            CA_RECEIVEERROR => {
                let res = message::parse_error(lparam)?;
                (self.on_error)(res.as_str());
                Ok(())
            },
            CA_CUSTOM_EXECUTE_POSTED_COMMAND => {
                while let Some(cmd) = self.request_queue.pop_front() {
                    cmd.call_lib(self.hwnd.unwrap())?;
                }
                Ok(())
            }
            _ => Err(QvOpenApiError::WindowUnknownEventError {
                wparam,
            }),
        }
    }

    fn on_destroy(&mut self) {
        self.hwnd = None;
    }
}

fn post_message_to_window(
    hwnd: isize,
    msg: u32,
    wparam: u32,
    lparam: isize,
) {
    debug!("message {} posted to {}", msg, hwnd);
    unsafe {
        PostMessageA(HWND(hwnd), msg, WPARAM(wparam as usize), LPARAM(lparam));
    }
}
