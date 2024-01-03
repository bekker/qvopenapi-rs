use std::{
    collections::VecDeque,
    sync::{Arc, Mutex, RwLock},
};

use serde_json::to_string_pretty;

use crate::{error::*, models::*, window_mgr::message_const::*, *};

pub trait QvOpenApiRequest: Send + Sync {
    fn before_post(&self) -> Result<(), QvOpenApiError>;
    fn call_lib(&self, tr_index: i32, hwnd: isize) -> Result<(), QvOpenApiError>;
    fn get_tr_code(&self) -> &str;
}

pub trait AbstractQvOpenApiClient {
    fn get_handler(&self) -> Arc<QvOpenApiClientMessageHandler>;

    fn get_hwnd(&self) -> Option<isize>;

    fn set_hwnd(&self, new_hwnd: isize);

    fn on_connect(&self, callback: Box<dyn FnMut(&ConnectResponse) + Send>) {
        self.get_handler()
            .message_handler
            .lock()
            .unwrap()
            .on_connect = callback;
    }

    fn on_disconnect(&self, callback: Box<dyn FnMut() + Send>) {
        self.get_handler()
            .message_handler
            .lock()
            .unwrap()
            .on_disconnect = callback;
    }

    fn on_socket_error(&self, callback: Box<dyn FnMut() + Send>) {
        self.get_handler()
            .message_handler
            .lock()
            .unwrap()
            .on_socket_error = callback;
    }

    fn on_data(&self, callback: Box<dyn FnMut(&DataResponse) + Send>) {
        self.get_handler().message_handler.lock().unwrap().on_data = callback;
    }

    fn on_sise(&self, callback: Box<dyn FnMut(&DataResponse) + Send>) {
        self.get_handler().message_handler.lock().unwrap().on_sise = callback;
    }

    fn on_message(&self, callback: Box<dyn FnMut(&MessageResponse) + Send>) {
        self.get_handler()
            .message_handler
            .lock()
            .unwrap()
            .on_message = callback;
    }

    fn on_complete(&self, callback: Box<dyn FnMut(i32) + Send>) {
        self.get_handler()
            .message_handler
            .lock()
            .unwrap()
            .on_complete = callback;
    }

    fn on_error(&self, callback: Box<dyn FnMut(&ErrorResponse) + Send>) {
        self.get_handler().message_handler.lock().unwrap().on_error = callback;
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
        self.query(
            TR_INDEX_CONNECT,
            Arc::new(ConnectRequest {
                account_type,
                id: id.into(),
                password: password.into(),
                cert_password: cert_password.into(),
            }),
        )
    }

    fn disconnect(&self) -> Result<(), QvOpenApiError> {
        self.query(-1, Arc::new(DisconnectRequest {}))
    }

    fn query(&self, tr_index: i32, req: Arc<dyn QvOpenApiRequest>) -> Result<(), QvOpenApiError>;
}

impl AbstractQvOpenApiClient for QvOpenApiClient {
    fn get_handler(&self) -> Arc<QvOpenApiClientMessageHandler> {
        self.handler.clone()
    }

    fn get_hwnd(&self) -> Option<isize> {
        *self.handler.hwnd_lock.read().unwrap()
    }

    fn set_hwnd(&self, new_hwnd: isize) {
        let mut hwnd = self.handler.hwnd_lock.write().unwrap();
        *hwnd = Some(new_hwnd);
    }

    fn query(&self, tr_index: i32, req: Arc<dyn QvOpenApiRequest>) -> Result<(), QvOpenApiError> {
        req.before_post()?;
        let hwnd = self.handler.hwnd_lock.read().unwrap();
        let mut request_queue = self.handler.request_queue_lock.lock().unwrap();
        request_queue.push_back((tr_index, req));
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
    pub message_handler: Mutex<QvOpenApiClientMessageCallbacks>,
    request_queue_lock: Mutex<VecDeque<(i32, Arc<dyn QvOpenApiRequest>)>>,
}

impl QvOpenApiClientMessageHandler {
    pub fn new() -> QvOpenApiClientMessageHandler {
        QvOpenApiClientMessageHandler {
            hwnd_lock: RwLock::new(None),
            message_handler: Mutex::new(QvOpenApiClientMessageCallbacks {
                on_connect: Box::new(|_| {}),
                on_disconnect: Box::new(|| {}),
                on_socket_error: Box::new(|| {}),
                on_data: Box::new(|_| {}),
                on_sise: Box::new(|_| {}),
                on_message: Box::new(|_| {}),
                on_complete: Box::new(|_| {}),
                on_error: Box::new(|_| {}),
            }),
            request_queue_lock: Mutex::new(VecDeque::new()),
        }
    }
}

pub struct QvOpenApiClientMessageCallbacks {
    pub on_connect: Box<dyn FnMut(&ConnectResponse) + Send>,
    pub on_disconnect: Box<dyn FnMut() + Send>,
    pub on_socket_error: Box<dyn FnMut() + Send>,
    pub on_data: Box<dyn FnMut(&DataResponse) + Send>,
    pub on_sise: Box<dyn FnMut(&DataResponse) + Send>,
    pub on_message: Box<dyn FnMut(&MessageResponse) + Send>,
    pub on_complete: Box<dyn FnMut(i32) + Send>,
    pub on_error: Box<dyn FnMut(&ErrorResponse) + Send>,
}

impl QvOpenApiClient {
    pub fn new() -> Result<QvOpenApiClient, QvOpenApiError> {
        crate::init()?;
        Ok(QvOpenApiClient {
            handler: Arc::new(QvOpenApiClientMessageHandler::new()),
        })
    }
}

impl QvOpenApiClientMessageHandler {
    pub fn on_wmca_msg(
        &self,
        wparam: usize,
        lparam: isize,
    ) -> std::result::Result<(), QvOpenApiError> {
        debug!("on_wmca_msg {} {}", wparam, lparam);
        match u32::try_from(wparam).unwrap() {
            CA_CONNECTED => {
                let res = models::parse_connect(lparam)?;
                debug!("CA_CONNECT {}", to_string_pretty(&res)?);
                let mut handler = self.message_handler.lock().unwrap();
                (handler.on_connect)(&res);
                Ok(())
            }
            CA_DISCONNECTED => {
                debug!("CA_DISCONNECTED");
                let mut handler = self.message_handler.lock().unwrap();
                (handler.on_disconnect)();
                Ok(())
            }
            CA_SOCKETERROR => {
                debug!("CA_SOCKETERROR");
                let mut handler = self.message_handler.lock().unwrap();
                (handler.on_socket_error)();
                Ok(())
            }
            CA_RECEIVEDATA => {
                let res = models::parse_data(lparam)?;
                debug!(
                    "CA_RECEIVEDATA [TR{}] {}",
                    res.tr_index,
                    to_string_pretty(&res)?
                );
                let mut handler = self.message_handler.lock().unwrap();
                (handler.on_data)(&res);
                Ok(())
            }
            CA_RECEIVESISE => {
                let res = models::parse_sise(lparam)?;
                debug!(
                    "CA_RECEIVESISE [TR{}] {}",
                    res.tr_index,
                    to_string_pretty(&res)?
                );
                let mut handler = self.message_handler.lock().unwrap();
                (handler.on_sise)(&res);
                Ok(())
            }
            CA_RECEIVEMESSAGE => {
                let res = models::parse_message(lparam)?;
                debug!(
                    "CA_RECEIVEMESSAGE [TR{}] [{}] \"{}\"",
                    res.tr_index, res.msg_code, res.msg
                );
                let mut handler = self.message_handler.lock().unwrap();
                (handler.on_message)(&res);
                Ok(())
            }
            CA_RECEIVECOMPLETE => {
                let res = models::parse_complete(lparam)?;
                debug!("CA_RECEIVECOMPLETE [TR{}]", res);
                let mut handler = self.message_handler.lock().unwrap();
                (handler.on_complete)(res);
                Ok(())
            }
            CA_RECEIVEERROR => {
                let res = models::parse_error(lparam)?;
                debug!("CA_RECEIVEERROR [TR{}] \"{}\"", res.tr_index, res.error_msg);
                let mut handler = self.message_handler.lock().unwrap();
                (handler.on_error)(&res);
                Ok(())
            }
            CA_CUSTOM_EXECUTE_POSTED_COMMAND => {
                let mut request_queue = self.request_queue_lock.lock().unwrap();
                let hwnd = self.hwnd_lock.read().unwrap();
                while let Some((tr_index, cmd)) = request_queue.pop_front() {
                    cmd.call_lib(tr_index, hwnd.unwrap())?;
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
