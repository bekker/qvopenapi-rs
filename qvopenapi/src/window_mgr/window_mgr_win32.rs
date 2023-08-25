use log::*;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex, RwLock},
    time::Duration,
};
use windows::{
    core::*, Win32::Foundation::*, Win32::Graphics::Gdi::*,
    Win32::System::LibraryLoader::GetModuleHandleA, Win32::UI::WindowsAndMessaging::*,
};

use crate::{*, client::QvOpenApiClientMessageHandler};

lazy_static! {
    static ref MESSAGE_HANDLER_MAP_LOCK: RwLock<HashMap<isize, Arc<QvOpenApiClientMessageHandler>>> =
        RwLock::new(HashMap::new());
}

pub unsafe fn destroy_window(hwnd: isize) {
    DestroyWindow(HWND(hwnd));
}

pub unsafe fn post_message(hwnd: isize, msg: u32, wparam: u32, lparam: isize) {
    debug!("message {} posted to {}", msg, hwnd);
    PostMessageA(HWND(hwnd), msg, WPARAM(wparam as usize), LPARAM(lparam));
}

pub fn run_window_async(
    manager_lock: Arc<RwLock<WindowHelper>>,
    message_handler: Arc<QvOpenApiClientMessageHandler>,
) -> std::result::Result<isize, QvOpenApiError> {
    {
        let reader = manager_lock.read().unwrap();
        if reader.status != WindowStatus::Init {
            return Err(QvOpenApiError::WindowAlreadyCreatedError);
        }
    }
    {
        let mut writer = manager_lock.write().unwrap();
        let cloned_lock = manager_lock.clone();
        let cloned_handler = message_handler.clone();
        writer.thread = Some(std::thread::spawn(move || {
            run_window_sync(cloned_lock, cloned_handler)
        }));
    }

    while manager_lock.read().unwrap().status == WindowStatus::Init {
        std::thread::sleep(Duration::from_millis(10))
    }

    {
        let reader = manager_lock.read().unwrap();
        if reader.status == WindowStatus::Created {
            info!("Window created (hwnd: {})", reader.hwnd.unwrap());
            Ok(reader.hwnd.unwrap())
        } else {
            info!("WindowManagerStatus is not CREATED");
            Err(QvOpenApiError::WindowCreationError)
        }
    }
}

fn run_window_sync(
    manager_lock: Arc<RwLock<WindowHelper>>,
    message_handler: Arc<QvOpenApiClientMessageHandler>,
) -> std::result::Result<(), QvOpenApiError> {
    let hwnd;
    {
        info!("Window creating...");
        let mut manager = manager_lock.write().unwrap();
        if manager.status != WindowStatus::Init {
            info!("WindowManagerStatus is not INIT");
            return Err(QvOpenApiError::WindowAlreadyCreatedError);
        }
        let create_result = create_window();

        if create_result.is_err() {
            manager.status = WindowStatus::Error;
            info!("WindowManagerStatus is ERROR");
            return Err(QvOpenApiError::WindowCreationError);
        }

        hwnd = create_result.unwrap().0;
        manager.hwnd = Some(hwnd);
        manager.status = WindowStatus::Created;
    }
    {
        let mut message_handler_map = MESSAGE_HANDLER_MAP_LOCK.write().unwrap();
        message_handler_map.insert(hwnd, message_handler.clone());
    }
    info!("Starting message loop (hwnd: {})", hwnd);
    loop_message();
    {
        let mut message_handler_map = MESSAGE_HANDLER_MAP_LOCK.write().unwrap();
        message_handler_map.remove(&hwnd);
    }
    {
        let mut manager = manager_lock.write().unwrap();
        manager.status = WindowStatus::Destroyed;
        info!("Window destroyed");
        message_handler.on_destroy();
    }
    Ok(())
}

fn create_window() -> windows::core::Result<HWND> {
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
fn loop_message() {
    unsafe {
        let mut message = MSG::default();
        while GetMessageW(&mut message, None, 0, 0).0 == 1 {
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
                let handler_map = MESSAGE_HANDLER_MAP_LOCK.read().unwrap();
                let handler = handler_map.get(&hwnd.0).unwrap();
                let res = handler.on_wmca_msg(wparam.0, lparam.0);
                match res {
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
