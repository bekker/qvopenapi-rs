extern crate custom_error;
extern crate libloading;

use custom_error::custom_error;

custom_error!{pub QvOpenApiError
	WmcaDllNotLoadedError = "wmca.dll not loaded",
	WmcaDllLoadingError = "Failed to load wmca.dll",
}

impl From<libloading::Error> for QvOpenApiError {
    fn from(_e: libloading::Error) -> Self {
        QvOpenApiError::WmcaDllLoadingError
    }
}
