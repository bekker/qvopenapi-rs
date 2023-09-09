use std::{sync::{Arc, RwLock, Mutex}, collections::VecDeque};

use crate::{*, window_mgr::message_const::*};

pub trait QvOpenApiRequest: Send + Sync {
    fn before_post(&self) -> Result<(), QvOpenApiError>;
    fn call_lib(&self, hwnd: isize) -> Result<(), QvOpenApiError>;
    fn get_tr_code(&self) -> &str;
    fn get_tr_index(&self) -> i32;
}

pub trait AbstractQvOpenApiClient {
    fn get_handler(&self) -> Arc<QvOpenApiClientMessageHandler>;

    fn set_hwnd(&self, new_hwnd: isize);

    fn on_connect(&mut self, callback: Box<dyn Fn(Arc<ConnectResponse>) + Send + Sync>) {
        self.get_handler().message_handler.write().unwrap().on_connect = callback;
    }

    fn on_disconnect(&mut self, callback: fn()) {
        self.get_handler().message_handler.write().unwrap().on_disconnect = callback;
    }

    fn on_socket_error(&mut self, callback: fn()) {
        self.get_handler().message_handler.write().unwrap().on_socket_error = callback;
    }

    fn on_data(&mut self, callback: fn(&DataResponse)) {
        self.get_handler().message_handler.write().unwrap().on_data = callback;
    }

    fn on_sise(&mut self, callback: fn(&DataResponse)) {
        self.get_handler().message_handler.write().unwrap().on_sise = callback;
    }

    fn on_message(&mut self, callback: fn(&MessageResponse)) {
        self.get_handler().message_handler.write().unwrap().on_message = callback;
    }

    fn on_complete(&mut self, callback: fn(i32)) {
        self.get_handler().message_handler.write().unwrap().on_complete = callback;
    }

    fn on_error(&mut self, callback: fn(&ErrorResponse)) {
        self.get_handler().message_handler.write().unwrap().on_error = callback;
    }

    fn connect(
        &self,
        new_hwnd: isize,
        account_type: AccountType,
        id: &str,
        password: &str,
        cert_password: &str,
    ) -> Result<(), QvOpenApiError> {
        self.set_hwnd(new_hwnd);
        self.query(Arc::new(ConnectRequest {
            account_type,
            id: id.into(),
            password: password.into(),
            cert_password: cert_password.into(),
        }))
    }

    fn query(
        &self,
        req: Arc<dyn QvOpenApiRequest>
    ) -> Result<(), QvOpenApiError>;
}

impl AbstractQvOpenApiClient for QvOpenApiClient {
    fn get_handler(&self) -> Arc<QvOpenApiClientMessageHandler> {
        self.handler.clone()
    }

    fn set_hwnd(&self, new_hwnd: isize) {
        let mut hwnd = self.handler.hwnd_lock.write().unwrap();
        *hwnd = Some(new_hwnd);
    }

    fn query(&self, req: Arc<dyn QvOpenApiRequest>) -> Result<(), QvOpenApiError> {
        req.before_post()?;
        let hwnd = self.handler.hwnd_lock.read().unwrap();
        let mut request_queue = self.handler.request_queue_lock.lock().unwrap();
        request_queue.push_back(req);
        window_mgr::post_message_to_window(
            hwnd.unwrap(),
            WM_WMCAEVENT,
            CA_CUSTOM_EXECUTE_POSTED_COMMAND,
            0,
        );
        Ok(())
    }
}

pub struct QvOpenApiClient {
    handler: Arc<QvOpenApiClientMessageHandler>,
}

pub struct QvOpenApiClientMessageHandler {
    hwnd_lock: RwLock<Option<isize>>,
    pub message_handler: RwLock<QvOpenApiClientMessageCallbacks>,
    request_queue_lock: Mutex<VecDeque<Arc<dyn QvOpenApiRequest>>>,
}

impl QvOpenApiClientMessageHandler {
    pub fn new() -> QvOpenApiClientMessageHandler {
        QvOpenApiClientMessageHandler {
            hwnd_lock: RwLock::new(None),
            message_handler: RwLock::new(QvOpenApiClientMessageCallbacks {
                on_connect: Box::new(|_| {}),
                on_disconnect: || {},
                on_socket_error: || {},
                on_data: |_| {},
                on_sise: |_| {},
                on_message: |_| {},
                on_complete: |_| {},
                on_error: |_| {},
            }),
            request_queue_lock: Mutex::new(VecDeque::new()),
        }
    }
}

pub struct QvOpenApiClientMessageCallbacks {
    pub on_connect: Box<dyn Fn(Arc<ConnectResponse>) + Send + Sync>,
    pub on_disconnect: fn(),
    pub on_socket_error: fn(),
    pub on_data: fn(&DataResponse),
    pub on_sise: fn(&DataResponse),
    pub on_message: fn(&MessageResponse),
    pub on_complete: fn(tr_index: i32),
    pub on_error: fn(&ErrorResponse),
}

impl QvOpenApiClient {
    pub fn new() -> QvOpenApiClient {
        QvOpenApiClient {
            handler: Arc::new(QvOpenApiClientMessageHandler::new()),
        }
    }
}

impl QvOpenApiClientMessageHandler {
    pub fn on_wmca_msg(&self, wparam: usize, lparam: isize) -> std::result::Result<(), QvOpenApiError> {
        debug!("on_wmca_msg {} {}", wparam, lparam);
        let handler = self.message_handler.read().unwrap();
        match u32::try_from(wparam).unwrap() {
            CA_CONNECTED => {
                let res = models::parse_connect(lparam)?;
                info!(
                    "CA_CONNECT (\"{}\", \"{}\", \"{}\", \"{}\")",
                    res.login_timestamp, res.server_name, res.user_id, res.account_count
                );
                (handler.on_connect)(Arc::new(res));
                Ok(())
            }
            CA_DISCONNECTED => {
                info!(
                    "CA_DISCONNECTED"
                );
                (handler.on_disconnect)();
                Ok(())
            }
            CA_SOCKETERROR => {
                info!(
                    "CA_SOCKETERROR"
                );
                (handler.on_socket_error)();
                Ok(())
            }
            CA_RECEIVEDATA => {
                let res: DataResponse = models::parse_data(lparam)?;
                info!(
                    "CA_RECEIVEDATA [TR{}] {} {}",
                    res.tr_index, res.block_name, res.block_len
                );
                (handler.on_data)(&res);
                Ok(())
            }
            CA_RECEIVESISE => {
                let res = models::parse_sise(lparam)?;
                info!(
                    "CA_RECEIVESISE [TR{}] {} {}",
                    res.tr_index, res.block_name, res.block_len
                );
                (handler.on_sise)(&res);
                Ok(())
            }
            CA_RECEIVEMESSAGE => {
                let res = models::parse_message(lparam)?;
                info!(
                    "CA_RECEIVEMESSAGE [TR{}] [{}] \"{}\"",
                    res.tr_index, res.msg_code, res.msg
                );
                (handler.on_message)(&res);
                Ok(())
            }
            CA_RECEIVECOMPLETE => {
                let res = models::parse_complete(lparam)?;
                info!("CA_RECEIVECOMPLETE [TR{}]", res);
                (handler.on_complete)(res);
                Ok(())
            }
            CA_RECEIVEERROR => {
                let res = models::parse_error(lparam)?;
                info!("CA_RECEIVEERROR [TR{}] \"{}\"", res.tr_index, res.error_msg);
                (handler.on_error)(&res);
                Ok(())
            }
            CA_CUSTOM_EXECUTE_POSTED_COMMAND => {
                let mut request_queue = self.request_queue_lock.lock().unwrap();
                let hwnd = self.hwnd_lock.read().unwrap();
                while let Some(cmd) = request_queue.pop_front() {
                    cmd.call_lib(hwnd.unwrap())?;
                }
                Ok(())
            }
            _ => Err(QvOpenApiError::WindowUnknownEventError { wparam }),
        }
    }

    pub fn on_destroy(&self) {
        let mut hwnd = self.hwnd_lock.write().unwrap();
        *hwnd = None;
    }
}
