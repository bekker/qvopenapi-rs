use std::{
    sync::{Mutex, RwLock},
    thread::JoinHandle,
    time::Duration,
};
use windows::{
    core::*, Win32::Foundation::*, Win32::Graphics::Gdi::*,
    Win32::System::LibraryLoader::GetModuleHandleA, Win32::UI::WindowsAndMessaging::*,
};
use log::*;

use crate::QvOpenApiError;

pub const CA_WMCAEVENT: u32 = 0x7000;

pub const CA_CONNECTED: u32 = WM_USER + 110;
pub const CA_DISCONNECTED: u32 = WM_USER + 120;
pub const CA_SOCKETERROR: u32 = WM_USER + 130;
pub const CA_RECEIVEDATA: u32 = WM_USER + 210;
pub const CA_RECEIVESISE: u32 = WM_USER + 220;
pub const CA_RECEIVEMESSAGE: u32 = WM_USER + 230;
pub const CA_RECEIVECOMPLETE: u32 = WM_USER + 240;
pub const CA_RECEIVEERROR: u32 = WM_USER + 250;

pub struct WindowManager {
    pub hwnd: Option<isize>,
    pub status: WindowManagerStatus,
    pub thread: Option<JoinHandle<std::result::Result<(), QvOpenApiError>>>,
}

#[derive(PartialEq, Eq)]
pub enum WindowManagerStatus {
    INIT,
    CREATED,
    DESTROYED,
    ERROR,
}

impl Drop for WindowManager {
    fn drop(&mut self) {
        if self.hwnd.is_some() && self.status != WindowManagerStatus::DESTROYED {
            info!("Destroying window...");
            unsafe {
                DestroyWindow(HWND{0: self.hwnd.unwrap()});
            }
        }
        self.thread.take().map(JoinHandle::join);
    }
}

impl WindowManager {
    pub const fn new() -> Self {
        WindowManager {
            hwnd: None,
            status: WindowManagerStatus::INIT,
            thread: None,
        }
    }
}

pub fn run_window(
    manager_lock: &'static RwLock<WindowManager>,
) -> std::result::Result<(), QvOpenApiError> {
    {
        let mut manager = manager_lock.write().unwrap();
        manager.thread = Some(std::thread::spawn(|| {
            {
                info!("Window creating...");
                let mut manager = manager_lock.write().unwrap();
                if manager.status != WindowManagerStatus::INIT {
                    info!("WindowManagerStatus is not INIT");
                    return Err(QvOpenApiError::WindowCreationError);
                }
                let create_result = create_window();

                if create_result.is_err() {
                    manager.status = WindowManagerStatus::ERROR;
                    info!("WindowManagerStatus is ERROR");
                    return Err(QvOpenApiError::WindowCreationError);
                }

                manager.hwnd = Some(create_result.unwrap().0);
                manager.status = WindowManagerStatus::CREATED;
                info!("Window created (hwnd: {})", manager.hwnd.unwrap());
            }
            loop_message();
            {
                let mut manager = manager_lock.write().unwrap();
                manager.status = WindowManagerStatus::DESTROYED;
                info!("Window destroyed");
            }
            Ok(())
        }));
    }

    while manager_lock.read().unwrap().status == WindowManagerStatus::INIT {
        std::thread::sleep(Duration::from_millis(10))
    }

    if manager_lock.read().unwrap().status == WindowManagerStatus::CREATED {
        return Ok(());
    } else {
        info!("WindowManagerStatus is not CREATED");
        return Err(QvOpenApiError::WindowCreationError);
    }
}

pub fn create_window() -> windows::core::Result<HWND> {
    unsafe {
        let instance = GetModuleHandleA(None)?;
        debug_assert!(instance.0 != 0);

        let window_class_name = s!("qvopenapi");

        let window_class = WNDCLASSA {
            hCursor: LoadCursorW(None, IDC_ARROW)?,
            hInstance: instance,
            lpszClassName: window_class_name,
            style: WNDCLASS_STYLES::default(),
            hbrBackground: HBRUSH(GetStockObject(WHITE_BRUSH).0),
            lpfnWndProc: Some(wndproc),
            ..Default::default()
        };

        let atom = RegisterClassA(&window_class);
        debug_assert!(atom != 0);

        let hwnd = CreateWindowExA(
            WINDOW_EX_STYLE::default(),
            window_class_name,
            s!("qvopenapi"),
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
pub fn loop_message() {
    unsafe {
        let mut message = MSG::default();
        while GetMessageA(&mut message, None, 0, 0).into() {
            DispatchMessageA(&message);
        }
    }
}

extern "system" fn wndproc(hwnd: HWND, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    unsafe {
        match message {
            WM_PAINT => {
                info!("WM_PAINT");
                draw(hwnd);
                LRESULT(0)
            }
            WM_CLOSE => {
                if MessageBoxA(
                    hwnd,
                    s!("Are you sure to close qvopenapi?"),
                    s!("Close"),
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
            CA_WMCAEVENT => {
                info!("CA_WMCAEVENT {}", wparam.0);
                match on_wmca_event(wparam, lparam) {
                    Ok(()) => LRESULT(0),
                    Err(e) => {
                        error!("QvOpenApiError: {}", e);
                        LRESULT(0)
                    }
                }
            }
            _ => DefWindowProcA(hwnd, message, wparam, lparam),
        }
    }
}

fn on_wmca_event(message_type: WPARAM, lparam: LPARAM) -> std::result::Result<(), QvOpenApiError> {
    match u32::try_from(message_type.0).unwrap() {
        CA_CONNECTED => Err(QvOpenApiError::EventUnimplementedError { event: String::from("CA_CONNECTED") }),
        CA_DISCONNECTED => Err(QvOpenApiError::EventUnimplementedError { event: String::from("CA_DISCONNECTED") }),
        CA_SOCKETERROR => Err(QvOpenApiError::EventUnimplementedError { event: String::from("CA_SOCKETERROR") }),
        CA_RECEIVEDATA => Err(QvOpenApiError::EventUnimplementedError { event: String::from("CA_RECEIVEDATA") }),
        CA_RECEIVESISE => Err(QvOpenApiError::EventUnimplementedError { event: String::from("CA_RECEIVESISE") }),
        CA_RECEIVEMESSAGE => Err(QvOpenApiError::EventUnimplementedError { event: String::from("CA_RECEIVEMESSAGE") }),
        CA_RECEIVECOMPLETE => Err(QvOpenApiError::EventUnimplementedError { event: String::from("CA_RECEIVECOMPLETE") }),
        CA_RECEIVEERROR => Err(QvOpenApiError::EventUnimplementedError { event: String::from("CA_RECEIVEERROR") }),
        _ => Err(QvOpenApiError::WindowUnknownEventError { wparam: message_type.0 })
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
