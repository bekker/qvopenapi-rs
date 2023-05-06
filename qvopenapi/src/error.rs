extern crate custom_error;
extern crate libloading;

use custom_error::custom_error;

custom_error! {#[derive(Clone)] pub QvOpenApiError
    WmcaDllNotLoadedError = "wmca.dll not loaded",
    WmcaDllLoadingError = "Failed to load wmca.dll",
    WindowCreationError = "Failed to create a window",
    WindowNotCreatedError = "Window not created",
    WindowUnknownEventError{ wparam: usize } = "Unknown event {wparam}",
    EventUnimplementedError{ event: String } = "Unimplemented event {event}",
    ReturnCodeError{ code: i32 } = "Return code {code}",
    NotConnectedError = "Not connected",
    QvApiMessageError{ message_code: String, message: String } = "[{message_code}] {message}",
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
