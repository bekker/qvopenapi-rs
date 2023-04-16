extern crate custom_error;
extern crate libloading;

use custom_error::custom_error;

custom_error! {pub QvOpenApiError
    WmcaDllNotLoadedError = "wmca.dll not loaded",
    WmcaDllLoadingError = "Failed to load wmca.dll",
    WindowCreationError = "Failed to create a window",
    WindowNotCreatedError = "Window not created",
}

impl From<libloading::Error> for QvOpenApiError {
    fn from(_e: libloading::Error) -> Self {
        QvOpenApiError::WmcaDllLoadingError
    }
}

impl From<windows::core::Error> for QvOpenApiError {
    fn from(_e: windows::core::Error) -> Self {
        QvOpenApiError::WindowCreationError
    }
}
