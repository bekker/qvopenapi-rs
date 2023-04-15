use qvopenapi_sys::WmcaLib;

extern crate qvopenapi_sys;

mod error;
pub use error::*;

mod window;

static mut WMCA_LIB: Option<WmcaLib> = None;

pub fn init() -> Result<(), QvOpenApiError> {
	load_lib()?;
	window::create_window();
	Ok(())
}

fn load_lib() -> Result<(), QvOpenApiError> {
	unsafe {
		println!("Loading wmca.dll");
		WMCA_LIB = Some(qvopenapi_sys::load_lib()?);
		println!("Loaded wmca.dll");
		Ok(())
	}
}

fn get_lib() -> Result<&'static WmcaLib, QvOpenApiError> {
	unsafe {
		match WMCA_LIB {
			Some(ref wmca_lib) => Ok(wmca_lib),
			None => Err(QvOpenApiError::WmcaDllNotLoadedError)
		}
	}
}

pub fn is_connected() -> Result<bool, QvOpenApiError> {
	Ok((get_lib()?.is_connected)() != 0)
}
