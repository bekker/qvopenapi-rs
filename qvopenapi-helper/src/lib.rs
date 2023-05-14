extern crate qvopenapi;
#[macro_use]
extern crate lazy_static;

mod window_mgr;

use std::sync::Arc;

use qvopenapi::*;

pub fn connect_with_default_window(
    connect_request: Arc<ConnectRequest>,
    on_window_destroy: fn(),
) -> Result<Arc<QvOpenApiClient>, QvOpenApiError> {
    qvopenapi::init()?;
    let client = Arc::new(QvOpenApiClient::new());

    {
        let mut writer = window_mgr::WINDOW_MANAGER_LOCK.write().unwrap();
        writer.on_destroy = on_window_destroy;
        writer.on_wmca_msg = &client.parse_wmca_msg;
    }
    window_mgr::run_window_async(&window_mgr::WINDOW_MANAGER_LOCK)?;
    
    let hwnd = window_mgr::get_hwnd()?;
    client.connect(hwnd, connect_request)?;

    Ok(client)
}
