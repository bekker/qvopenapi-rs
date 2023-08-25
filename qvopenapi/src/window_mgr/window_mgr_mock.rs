use std::sync::{Arc, RwLock};

use crate::{*, client::QvOpenApiClientMessageHandler};

pub fn run_window_async(
    _manager_lock: Arc<RwLock<WindowHelper>>,
    _message_handler: Arc<QvOpenApiClientMessageHandler>,
) -> std::result::Result<isize, QvOpenApiError> {
    unimplemented!()
}

pub unsafe fn destroy_window(_hwnd: isize) {
    unimplemented!()
}

pub unsafe fn post_message(_hwnd: isize, _msg: u32, _wparam: u32, _lparam: isize) {
    unimplemented!()
}
