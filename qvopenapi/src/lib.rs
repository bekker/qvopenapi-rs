use qvopenapi_sys::WmcaLib;

extern crate qvopenapi_sys;

mod error;
pub use error::*;

static mut WMCA_LIB: Option<WmcaLib> = None;

pub fn is_connected() -> Result<bool, QvOpenApiError> {
	Ok((get_lib()?.is_connected)() != 0)
}

fn get_lib() -> Result<&'static WmcaLib, QvOpenApiError> {
	unsafe {
		match WMCA_LIB {
			Some(ref wmca_lib) => Ok(wmca_lib),
			None => Err(QvOpenApiError::WmcaDllNotLoadedError)
		}
	}
}

pub fn load_lib() -> Result<(), QvOpenApiError> {
	unsafe {
		WMCA_LIB = Some(qvopenapi_sys::load_lib()?);
		Ok(())
	}
}
