use std::{
    sync::{Mutex, RwLock},
    thread::JoinHandle,
    time::Duration,
};
use windows::{
    core::*, Win32::Foundation::*, Win32::Graphics::Gdi::*,
    Win32::System::LibraryLoader::GetModuleHandleA, Win32::UI::WindowsAndMessaging::*,
};

use crate::QvOpenApiError;

pub struct WindowManager {
    pub hwnd: Option<HWND>,
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
            println!("Destroying window...");
            unsafe {
                DestroyWindow(self.hwnd.unwrap());
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
    manager: &'static RwLock<WindowManager>,
) -> std::result::Result<(), QvOpenApiError> {
    {
        let mut writer = manager.write().unwrap();
        (*writer).thread = Some(std::thread::spawn(|| {
            {
                println!("Window creating...");
                let mut writer = manager.write().unwrap();
                if (*writer).status != WindowManagerStatus::INIT {
                    println!("WindowManagerStatus is not INIT");
                    return Err(QvOpenApiError::WindowCreationError);
                }
                let create_result = create_window();

                if create_result.is_err() {
                    (*writer).status = WindowManagerStatus::ERROR;
                    println!("WindowManagerStatus is ERROR");
                    return Err(QvOpenApiError::WindowCreationError);
                }

                (*writer).hwnd = Some(create_result.unwrap());
                (*writer).status = WindowManagerStatus::CREATED;
                println!("Window created");
            }
            loop_message();
            {
                let mut writer = manager.write().unwrap();
                (*writer).status = WindowManagerStatus::DESTROYED;
                println!("Window destroyed");
            }
            Ok(())
        }));
    }

    while manager.read().unwrap().status == WindowManagerStatus::INIT {
        std::thread::sleep(Duration::from_millis(10))
    }

    if manager.read().unwrap().status == WindowManagerStatus::CREATED {
        return Ok(());
    } else {
        println!("WindowManagerStatus is not CREATED");
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
                println!("WM_PAINT");
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
                println!("WM_DESTROY");
                PostQuitMessage(0);
                LRESULT(0)
            }
            _ => DefWindowProcA(hwnd, message, wparam, lparam),
        }
    }
}

lazy_static! {
    static ref DRAW_TEXT: Mutex<Vec<u16>> = Mutex::new(
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
        DRAW_TEXT.lock().unwrap().as_mut(),
        &mut rc,
        DT_CENTER | DT_VCENTER | DT_SINGLELINE,
    );
    ReleaseDC(hwnd, dc);
}
