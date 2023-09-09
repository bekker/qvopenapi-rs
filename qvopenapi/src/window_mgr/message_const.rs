#[cfg(target_os = "windows")]
use windows::Win32::UI::WindowsAndMessaging::WM_USER;

#[cfg(not(target_os = "windows"))]
pub const WM_USER: u32 = 0x0400;

pub const WM_WMCAEVENT: u32 = WM_USER + 8400;

pub const CA_CUSTOM_EXECUTE_POSTED_COMMAND: u32 = WM_USER + 8410;
pub const CA_CONNECTED: u32 = WM_USER + 110;
pub const CA_DISCONNECTED: u32 = WM_USER + 120;
pub const CA_SOCKETERROR: u32 = WM_USER + 130;
pub const CA_RECEIVEDATA: u32 = WM_USER + 210;
pub const CA_RECEIVESISE: u32 = WM_USER + 220;
pub const CA_RECEIVEMESSAGE: u32 = WM_USER + 230;
pub const CA_RECEIVECOMPLETE: u32 = WM_USER + 240;
pub const CA_RECEIVEERROR: u32 = WM_USER + 250;
