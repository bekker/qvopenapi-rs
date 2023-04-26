/**
 * https://github.com/rust-lang/rust/issues/79609
 * To bypass missing 'dwarf' exception handling on mingw i686 installations,
 * set panic = abort and provide mock unwind symbol to the linker
 */
#[cfg(feature = "disable-unwind")]
#[no_mangle]
pub extern "stdcall" fn _Unwind_Resume() {}

extern crate libloading;
extern crate windows_sys;

use std::fmt;

use libc::{c_char, c_int};
#[cfg(unix)]
use libloading::os::unix::Symbol as RawSymbol;
#[cfg(windows)]
use libloading::os::windows::Symbol as RawSymbol;
use libloading::{Library, Symbol};
use windows_sys::Win32::Foundation::*;

type BOOL = c_int;
type DWORD = u32;

type WmcaLoad = extern "stdcall" fn() -> BOOL;
type WmcaFree = extern "stdcall" fn() -> BOOL;
type WmcaSetServer = extern "stdcall" fn(*const c_char) -> BOOL;
type WmcaSetPort = extern "stdcall" fn(c_int) -> BOOL;
type WmcaIsConnected = extern "stdcall" fn() -> BOOL;
type WmcaConnect = extern "stdcall" fn(
    HWND,
    DWORD,
    c_char,
    c_char,
    *const c_char,
    *const c_char,
    *const c_char,
) -> BOOL;
type WmcaDisconnect = extern "stdcall" fn() -> BOOL;
type WmcaTransact =
    extern "stdcall" fn(HWND, c_int, *const c_char, *const c_char, c_int, c_int, c_int) -> BOOL;
type WmcaQuery =
    extern "stdcall" fn(HWND, c_int, *const c_char, *const c_char, c_int, c_int) -> BOOL;
type WmcaRequest =
    extern "stdcall" fn(HWND, c_int, *const c_char, *const c_char, c_int, *const c_char) -> BOOL;
type WmcaAttach = extern "stdcall" fn(HWND, *const c_char, *const c_char, c_int, c_int) -> BOOL;
type WmcaDetach = extern "stdcall" fn(HWND, *const c_char, *const c_char, c_int, c_int) -> BOOL;
type WmcaDetachWindow = extern "stdcall" fn(HWND) -> BOOL;
type WmcaDetachAll = extern "stdcall" fn() -> BOOL;
type WmcaSetOption = extern "stdcall" fn(*const c_char, *const c_char) -> BOOL;
type WmcaSetAccountIndexPwd = extern "stdcall" fn(*const c_char, c_int, *const c_char) -> BOOL;
type WmcaSetOrderPwd = extern "stdcall" fn(*const c_char, *const c_char) -> BOOL;
type WmcaSetHashPwd = extern "stdcall" fn(*const c_char, *const c_char, *const c_char) -> BOOL;
type WmcaSetAccountNoPwd = extern "stdcall" fn(*const c_char, *const c_char, *const c_char) -> BOOL;
type WmcaSetAccountNoByIndex = extern "stdcall" fn(*const c_char, c_int) -> BOOL;

pub struct WmcaLib {
    pub library: Library,
    pub load: RawSymbol<WmcaLoad>,
    pub free: RawSymbol<WmcaFree>,
    pub set_server: RawSymbol<WmcaSetServer>,
    pub set_port: RawSymbol<WmcaSetPort>,
    pub is_connected: RawSymbol<WmcaIsConnected>,
    pub connect: RawSymbol<WmcaConnect>,
    pub disconnect: RawSymbol<WmcaDisconnect>,
    pub transact: RawSymbol<WmcaTransact>,
    pub query: RawSymbol<WmcaQuery>,
    pub request: RawSymbol<WmcaRequest>,
    pub attach: RawSymbol<WmcaAttach>,
    pub detach: RawSymbol<WmcaDetach>,
    pub detach_window: RawSymbol<WmcaDetachWindow>,
    pub detach_all: RawSymbol<WmcaDetachAll>,
    pub set_option: RawSymbol<WmcaSetOption>,
    pub set_account_index_pwd: RawSymbol<WmcaSetAccountIndexPwd>,
    pub set_order_pwd: RawSymbol<WmcaSetOrderPwd>,
    pub set_hash_pwd: RawSymbol<WmcaSetHashPwd>,
    pub set_account_no_pwd: RawSymbol<WmcaSetAccountNoPwd>,
    pub set_account_no_by_index: RawSymbol<WmcaSetAccountNoByIndex>,
}

impl fmt::Debug for WmcaLib {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "WmcaLib")
    }
}

pub fn bind_lib() -> Result<WmcaLib, libloading::Error> {
    unsafe {
        let library = Library::new("wmca.dll")?;
        let load: Symbol<WmcaLoad> = library.get(b"wmcaLoad")?;
        let load = load.into_raw();
        let free: Symbol<WmcaFree> = library.get(b"wmcaFree")?;
        let free = free.into_raw();
        let set_server: Symbol<WmcaSetServer> = library.get(b"wmcaSetServer")?;
        let set_server = set_server.into_raw();
        let set_port: Symbol<WmcaSetPort> = library.get(b"wmcaSetPort")?;
        let set_port = set_port.into_raw();
        let is_connected: Symbol<WmcaIsConnected> = library.get(b"wmcaIsConnected")?;
        let is_connected = is_connected.into_raw();
        let connect: Symbol<WmcaConnect> = library.get(b"wmcaConnect")?;
        let connect = connect.into_raw();
        let disconnect: Symbol<WmcaDisconnect> = library.get(b"wmcaDisconnect")?;
        let disconnect = disconnect.into_raw();
        let transact: Symbol<WmcaTransact> = library.get(b"wmcaTransact")?;
        let transact = transact.into_raw();
        let query: Symbol<WmcaQuery> = library.get(b"wmcaQuery")?;
        let query = query.into_raw();
        let request: Symbol<WmcaRequest> = library.get(b"wmcaRequest")?;
        let request = request.into_raw();
        let attach: Symbol<WmcaAttach> = library.get(b"wmcaAttach")?;
        let attach = attach.into_raw();
        let detach: Symbol<WmcaDetach> = library.get(b"wmcaDetach")?;
        let detach = detach.into_raw();
        let detach_window: Symbol<WmcaDetachWindow> = library.get(b"wmcaDetachWindow")?;
        let detach_window = detach_window.into_raw();
        let detach_all: Symbol<WmcaDetachAll> = library.get(b"wmcaDetachAll")?;
        let detach_all = detach_all.into_raw();
        let set_option: Symbol<WmcaSetOption> = library.get(b"wmcaSetOption")?;
        let set_option = set_option.into_raw();
        let set_account_index_pwd: Symbol<WmcaSetAccountIndexPwd> =
            library.get(b"wmcaSetAccountIndexPwd")?;
        let set_account_index_pwd = set_account_index_pwd.into_raw();
        let set_order_pwd: Symbol<WmcaSetOrderPwd> = library.get(b"wmcaSetOrderPwd")?;
        let set_order_pwd = set_order_pwd.into_raw();
        let set_hash_pwd: Symbol<WmcaSetHashPwd> = library.get(b"wmcaSetHashPwd")?;
        let set_hash_pwd = set_hash_pwd.into_raw();
        let set_account_no_pwd: Symbol<WmcaSetAccountNoPwd> =
            library.get(b"wmcaSetAccountNoPwd")?;
        let set_account_no_pwd = set_account_no_pwd.into_raw();
        let set_account_no_by_index: Symbol<WmcaSetAccountNoByIndex> =
            library.get(b"wmcaSetAccountNoByIndex")?;
        let set_account_no_by_index = set_account_no_by_index.into_raw();
        Ok(WmcaLib {
            library,
            load,
            free,
            set_server,
            set_port,
            is_connected,
            connect,
            disconnect,
            transact,
            query,
            request,
            attach,
            detach,
            detach_window,
            detach_all,
            set_option,
            set_account_index_pwd,
            set_order_pwd,
            set_hash_pwd,
            set_account_no_pwd,
            set_account_no_by_index,
        })
    }
}
