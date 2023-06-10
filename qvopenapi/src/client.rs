use std::{sync::{Arc, RwLock, Mutex}, collections::VecDeque};

use crate::*;

pub trait QvOpenApiClient {
    fn get_handler(&self) -> Arc<QvOpenApiClientMessageHandler>;
}

impl From<&dyn QvOpenApiClient> for Arc<QvOpenApiClientMessageHandler> {
    fn from(value: &dyn QvOpenApiClient) -> Arc<QvOpenApiClientMessageHandler> {
        value.get_handler()
    }
}

impl QvOpenApiClient for SimpleQvOpenApiClient {
    fn get_handler(&self) -> Arc<QvOpenApiClientMessageHandler> {
        self.handler.clone()
    }
}

pub struct SimpleQvOpenApiClient {
    handler: Arc<QvOpenApiClientMessageHandler>,
}

pub struct QvOpenApiClientMessageHandler {
    hwnd_lock: RwLock<Option<isize>>,
    message_handler: RwLock<QvOpenApiClientMessageCallbacks>,
    request_queue_lock: Mutex<VecDeque<Arc<dyn QvOpenApiRequest>>>,
}

struct QvOpenApiClientMessageCallbacks {
    on_connect: fn(&ConnectResponse),
    on_disconnect: fn(),
    on_socket_error: fn(),
    on_data: fn(&DataResponse),
    on_sise: fn(&DataResponse),
    on_message: fn(&MessageResponse),
    on_complete: fn(tr_index: i32),
    on_error: fn(&ErrorResponse),
}

impl SimpleQvOpenApiClient {
    pub fn new() -> SimpleQvOpenApiClient {
        SimpleQvOpenApiClient {
            handler: Arc::new(QvOpenApiClientMessageHandler {
                hwnd_lock: RwLock::new(None),
                message_handler: RwLock::new(QvOpenApiClientMessageCallbacks {
                    on_connect: |_| {},
                    on_disconnect: || {},
                    on_socket_error: || {},
                    on_data: |_| {},
                    on_sise: |_| {},
                    on_message: |_| {},
                    on_complete: |_| {},
                    on_error: |_| {},
                }),
                request_queue_lock: Mutex::new(VecDeque::new()),
            }),
        }
    }

    pub fn on_connect(&mut self, callback: fn(&ConnectResponse)) {
        self.handler.message_handler.write().unwrap().on_connect = callback;
    }

    pub fn on_disconnect(&mut self, callback: fn()) {
        self.handler.message_handler.write().unwrap().on_disconnect = callback;
    }

    pub fn on_socket_error(&mut self, callback: fn()) {
        self.handler.message_handler.write().unwrap().on_socket_error = callback;
    }

    pub fn on_data(&mut self, callback: fn(&DataResponse)) {
        self.handler.message_handler.write().unwrap().on_data = callback;
    }

    pub fn on_sise(&mut self, callback: fn(&DataResponse)) {
        self.handler.message_handler.write().unwrap().on_sise = callback;
    }

    pub fn on_message(&mut self, callback: fn(&MessageResponse)) {
        self.handler.message_handler.write().unwrap().on_message = callback;
    }

    pub fn on_complete(&mut self, callback: fn(i32)) {
        self.handler.message_handler.write().unwrap().on_complete = callback;
    }

    pub fn on_error(&mut self, callback: fn(&ErrorResponse)) {
        self.handler.message_handler.write().unwrap().on_error = callback;
    }

    pub fn connect(
        &self,
        new_hwnd: isize,
        account_type: AccountType,
        id: &str,
        password: &str,
        cert_password: &str,
    ) -> Result<(), QvOpenApiError> {
        {
            let mut hwnd = self.handler.hwnd_lock.write().unwrap();
            *hwnd = Some(new_hwnd);
        }
        self.post_command(Arc::new(ConnectRequest {
            account_type,
            id: id.into(),
            password: password.into(),
            cert_password: cert_password.into(),
        }))
    }

    pub fn get_balance(
        &self,
        tr_index: i32,
        account_index: i32,
        balance_type: char,
    ) -> Result<(), QvOpenApiError> {
        self.query(make_c8201_request(tr_index, account_index, balance_type)?)
    }

    pub fn query<T: Send + Sync + 'static>(
        &self,
        req: QueryRequest<T>
    ) -> Result<(), QvOpenApiError> {
        self.post_command(Arc::new(req))
    }

    fn post_command(&self, command: Arc<dyn QvOpenApiRequest>) -> Result<(), QvOpenApiError> {
        command.before_post()?;
        let hwnd = self.handler.hwnd_lock.read().unwrap();
        let mut request_queue = self.handler.request_queue_lock.lock().unwrap();
        request_queue.push_back(command);
        post_message_to_window(
            hwnd.unwrap(),
            WM_WMCAEVENT,
            CA_CUSTOM_EXECUTE_POSTED_COMMAND,
            0,
        );
        Ok(())
    }
}

impl QvOpenApiClientMessageHandler {
    pub fn on_wmca_msg(&self, wparam: usize, lparam: isize) -> std::result::Result<(), QvOpenApiError> {
        debug!("on_wmca_msg {} {}", wparam, lparam);
        let handler = self.message_handler.read().unwrap();
        match u32::try_from(wparam).unwrap() {
            CA_CONNECTED => {
                let res = message::parse_connect(lparam)?;
                info!(
                    "CA_CONNECT (\"{}\", \"{}\", \"{}\", \"{}\")",
                    res.login_datetime, res.server_name, res.user_id, res.account_count
                );
                (handler.on_connect)(&res);
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
                let res = message::parse_data(lparam)?;
                info!(
                    "CA_RECEIVEDATA [TR{}] {} {}",
                    res.tr_index, res.block_name, res.block_len
                );
                (handler.on_data)(&res);
                Ok(())
            }
            CA_RECEIVESISE => {
                let res = message::parse_sise(lparam)?;
                info!(
                    "CA_RECEIVESISE [TR{}] {} {}",
                    res.tr_index, res.block_name, res.block_len
                );
                (handler.on_sise)(&res);
                Ok(())
            }
            CA_RECEIVEMESSAGE => {
                let res = message::parse_message(lparam)?;
                info!(
                    "CA_RECEIVEMESSAGE [TR{}] [{}] \"{}\"",
                    res.tr_index, res.msg_code, res.msg
                );
                (handler.on_message)(&res);
                Ok(())
            }
            CA_RECEIVECOMPLETE => {
                let res = message::parse_complete(lparam)?;
                info!("CA_RECEIVECOMPLETE [TR{}]", res);
                (handler.on_complete)(res);
                Ok(())
            }
            CA_RECEIVEERROR => {
                let res = message::parse_error(lparam)?;
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

fn post_message_to_window(hwnd: isize, msg: u32, wparam: u32, lparam: isize) {
    debug!("message {} posted to {}", msg, hwnd);
    unsafe {
        PostMessageA(HWND(hwnd), msg, WPARAM(wparam as usize), LPARAM(lparam));
    }
}
