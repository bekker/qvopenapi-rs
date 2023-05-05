use encoding::{all::WINDOWS_949, DecoderTrap, Encoding};
use log::*;
use std::{
    ffi::{c_char, c_int, CStr},
    sync::{Mutex, RwLock},
    thread::JoinHandle,
    time::Duration,
};
use windows::{
    core::*, Win32::Foundation::*, Win32::Graphics::Gdi::*,
    Win32::System::LibraryLoader::GetModuleHandleA, Win32::UI::WindowsAndMessaging::*,
};

use crate::*;

pub const WM_WMCAEVENT: u32 = WM_USER + 8400;

pub const CA_CONNECTED: u32 = WM_USER + 110;
pub const CA_DISCONNECTED: u32 = WM_USER + 120;
pub const CA_SOCKETERROR: u32 = WM_USER + 130;
pub const CA_RECEIVEDATA: u32 = WM_USER + 210;
pub const CA_RECEIVESISE: u32 = WM_USER + 220;
pub const CA_RECEIVEMESSAGE: u32 = WM_USER + 230;
pub const CA_RECEIVECOMPLETE: u32 = WM_USER + 240;
pub const CA_RECEIVEERROR: u32 = WM_USER + 250;

pub const WM_CUSTOMEVENT: u32 = WM_USER + 8410;

pub const CA_COMMAND: u32 = WM_USER + 110;

pub static WINDOW_MANAGER_LOCK: RwLock<WindowManager> = RwLock::new(WindowManager::new());

pub struct WindowManager {
    pub hwnd: Option<isize>,
    pub status: WindowManagerStatus,
    pub thread: Option<JoinHandle<std::result::Result<(), QvOpenApiError>>>,
}

#[derive(PartialEq, Eq)]
pub enum WindowManagerStatus {
    Init,
    Created,
    Destroyed,
    Error,
}

impl Drop for WindowManager {
    fn drop(&mut self) {
        if self.hwnd.is_some() && self.status != WindowManagerStatus::Destroyed {
            info!("Destroying window...");
            unsafe {
                DestroyWindow(HWND(self.hwnd.unwrap()));
            }
        }
        self.thread.take().map(JoinHandle::join);
    }
}

impl WindowManager {
    pub const fn new() -> Self {
        WindowManager {
            hwnd: None,
            status: WindowManagerStatus::Init,
            thread: None,
        }
    }
}

pub fn run_window_async(
    manager_lock: &'static RwLock<WindowManager>,
) -> std::result::Result<(), QvOpenApiError> {
    {
        let mut manager = manager_lock.write().unwrap();
        manager.thread = Some(std::thread::spawn(move || run_window_sync(manager_lock)));
    }

    while manager_lock.read().unwrap().status == WindowManagerStatus::Init {
        std::thread::sleep(Duration::from_millis(10))
    }

    if manager_lock.read().unwrap().status == WindowManagerStatus::Created {
        Ok(())
    } else {
        info!("WindowManagerStatus is not CREATED");
        Err(QvOpenApiError::WindowCreationError)
    }
}

pub fn run_window_sync(
    manager_lock: &'static RwLock<WindowManager>,
) -> std::result::Result<(), QvOpenApiError> {
    let hwnd;
    {
        info!("Window creating...");
        let mut manager = manager_lock.write().unwrap();
        if manager.status != WindowManagerStatus::Init {
            info!("WindowManagerStatus is not INIT");
            return Err(QvOpenApiError::WindowCreationError);
        }
        let create_result = create_window();

        if create_result.is_err() {
            manager.status = WindowManagerStatus::Error;
            info!("WindowManagerStatus is ERROR");
            return Err(QvOpenApiError::WindowCreationError);
        }

        hwnd = create_result.unwrap().0;
        manager.hwnd = Some(hwnd);
        manager.status = WindowManagerStatus::Created;
        info!("Window created (hwnd: {})", manager.hwnd.unwrap());
    }
    loop_message(hwnd);
    {
        let mut manager = manager_lock.write().unwrap();
        manager.status = WindowManagerStatus::Destroyed;
        info!("Window destroyed");
    }
    Ok(())
}

pub fn create_window() -> windows::core::Result<HWND> {
    unsafe {
        let instance = GetModuleHandleA(None)?;
        debug_assert!(instance.0 != 0);

        let window_class_name = w!("qvopenapi");

        let window_class = WNDCLASSW {
            hCursor: LoadCursorW(None, IDC_ARROW)?,
            hInstance: instance,
            lpszClassName: window_class_name,
            style: WNDCLASS_STYLES::default(),
            hbrBackground: HBRUSH(GetStockObject(WHITE_BRUSH).0),
            lpfnWndProc: Some(wndproc),
            ..Default::default()
        };

        let atom = RegisterClassW(&window_class);
        debug_assert!(atom != 0);

        let hwnd = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            window_class_name,
            w!("qvopenapi"),
            WS_OVERLAPPED | WS_MINIMIZEBOX | WS_SYSMENU | WS_VISIBLE | WS_BORDER,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            400,
            300,
            None,
            None,
            instance,
            None,
        );
        Ok(hwnd)
    }
}

/**
 * Must be called from the same thread as create_window()
 */
pub fn loop_message(hwnd: isize) {
    unsafe {
        let mut message = MSG::default();
        while GetMessageW(&mut message, HWND(hwnd), 0, 0).0 == 1 {
            TranslateMessage(&message);
            DispatchMessageW(&message);
        }
    }
}

#[no_mangle]
extern "system" fn wndproc(hwnd: HWND, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    unsafe {
        match message {
            WM_PAINT => {
                info!("WM_PAINT");
                draw(hwnd);
                LRESULT(0)
            }
            WM_CLOSE => {
                if MessageBoxW(
                    hwnd,
                    w!("Are you sure to close qvopenapi?"),
                    w!("Close"),
                    MB_ICONQUESTION | MB_YESNO,
                ) == IDYES
                {
                    DestroyWindow(hwnd);
                }
                LRESULT(0)
            }
            WM_DESTROY => {
                info!("WM_DESTROY");
                PostQuitMessage(0);
                LRESULT(0)
            }
            WM_WMCAEVENT => {
                debug!("WM_WMCAEVENT {}", wparam.0);
                match on_wmca_event(wparam.0, lparam.0) {
                    Ok(()) => LRESULT(0),
                    Err(e) => {
                        error!("QvOpenApiError: {}", e);
                        LRESULT(0)
                    }
                }
            }
            WM_CUSTOMEVENT => {
                debug!("WM_CUSTOMEVENT {}", wparam.0);
                match on_custom_event(wparam.0, lparam.0) {
                    Ok(()) => LRESULT(0),
                    Err(e) => {
                        error!("QvOpenApiError: {}", e);
                        LRESULT(0)
                    }
                }
            }
            _ => {
                //debug!("DefWindowProcW {} {} {}", message, wparam.0, lparam.0);
                DefWindowProcW(hwnd, message, wparam, lparam)
            }
        }
    }
}

fn on_wmca_event(message_type: usize, lparam: isize) -> std::result::Result<(), QvOpenApiError> {
    match u32::try_from(message_type).unwrap() {
        CA_CONNECTED => Err(QvOpenApiError::EventUnimplementedError {
            event: String::from("CA_CONNECTED"),
        }),
        CA_DISCONNECTED => Err(QvOpenApiError::EventUnimplementedError {
            event: String::from("CA_DISCONNECTED"),
        }),
        CA_SOCKETERROR => Err(QvOpenApiError::EventUnimplementedError {
            event: String::from("CA_SOCKETERROR"),
        }),
        CA_RECEIVEDATA => Err(QvOpenApiError::EventUnimplementedError {
            event: String::from("CA_RECEIVEDATA"),
        }),
        CA_RECEIVESISE => Err(QvOpenApiError::EventUnimplementedError {
            event: String::from("CA_RECEIVESISE"),
        }),
        CA_RECEIVEMESSAGE => on_receive_message(lparam),
        CA_RECEIVECOMPLETE => Err(QvOpenApiError::EventUnimplementedError {
            event: String::from("CA_RECEIVECOMPLETE"),
        }),
        CA_RECEIVEERROR => Err(QvOpenApiError::EventUnimplementedError {
            event: String::from("CA_RECEIVEERROR"),
        }),
        _ => Err(QvOpenApiError::WindowUnknownEventError {
            wparam: message_type,
        }),
    }
}

fn on_custom_event(message_type: usize, lparam: isize) -> std::result::Result<(), QvOpenApiError> {
    match u32::try_from(message_type).unwrap() {
        CA_COMMAND => command::execute_command(),
        _ => Err(QvOpenApiError::WindowUnknownEventError {
            wparam: message_type,
        }),
    }
}

#[repr(C)]
pub struct OutDataBlock<T> {
    pub tr_index: c_int,
    pub p_data: *const ReceivedData<T>,
}

#[repr(C)]
pub struct ReceivedData<T> {
    pub block_name: *const c_char,
    pub sz_data: *const T,
    pub len: c_int,
}

#[repr(C)]
pub struct MessageHeader {
    pub message_code: [c_char; 5],
    pub message: [c_char; 80],
}

fn on_receive_message(lparam: isize) -> std::result::Result<(), QvOpenApiError> {
    let data_block = lparam as *const OutDataBlock<MessageHeader>;
    unsafe {
        let msg_header = (*(*data_block).p_data).sz_data;
        let message_code = from_cp949(&(*msg_header).message_code);
        let message = from_cp949(&(*msg_header).message);
        info!("CA_RECEIVEMESSAGE [{}] \"{}\"", message_code, message);
    }

    Ok(())
}

/**
 * 문자열 끝에 null이 없을 수도, 있을 수도 있음
 */
fn from_cp949(src: &[c_char]) -> String {
    let mut cloned: Vec<u8> = vec![];
    for s in src.iter() {
        // null을 만나면 여기까지만
        if *s == 0 {
            break;
        }
        cloned.push(*s as u8);
    }
    WINDOWS_949
        .decode(cloned.as_slice(), DecoderTrap::Replace)
        .unwrap()
}

/**
 * 문자열 끝에 null이 있음
 */
fn from_cp949_ptr(src: *const c_char) -> String {
    unsafe {
        let cstr: &CStr = CStr::from_ptr(src);
        WINDOWS_949
            .decode(cstr.to_bytes(), DecoderTrap::Replace)
            .unwrap()
    }
}

// Somehow DrawTextW wants mutable string (though it doesn't seem need one)
// So I made one for it
lazy_static! {
    static ref DRAW_TEXT_MUT: Mutex<Vec<u16>> = Mutex::new(
        String::from("qvopenapi")
            .encode_utf16()
            .collect::<Vec<u16>>()
    );
}

unsafe fn draw(hwnd: HWND) {
    ValidateRect(hwnd, None);
    let dc: HDC = GetDC(hwnd);
    let mut rc: RECT = RECT::default();
    GetClientRect(hwnd, &mut rc);
    DrawTextW(
        dc,
        DRAW_TEXT_MUT.lock().unwrap().as_mut(),
        &mut rc,
        DT_CENTER | DT_VCENTER | DT_SINGLELINE,
    );
    ReleaseDC(hwnd, dc);
}

pub fn get_hwnd() -> std::result::Result<isize, QvOpenApiError> {
    match WINDOW_MANAGER_LOCK.read().unwrap().hwnd {
        Some(hwnd) => Ok(hwnd),
        None => Err(QvOpenApiError::WindowNotCreatedError),
    }
}

pub fn send_message(
    msg: u32,
    wparam: usize,
    lparam: isize,
) -> std::result::Result<(), QvOpenApiError> {
    unsafe {
        SendMessageA(HWND(get_hwnd()?), msg, WPARAM(wparam), LPARAM(lparam));
    }
    Ok(())
}

pub fn post_message(
    msg: u32,
    wparam: usize,
    lparam: isize,
) -> std::result::Result<(), QvOpenApiError> {
    unsafe {
        PostMessageA(HWND(get_hwnd()?), msg, WPARAM(wparam), LPARAM(lparam));
    }
    Ok(())
}
