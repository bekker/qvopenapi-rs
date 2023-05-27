extern crate custom_error;
extern crate libloading;

use custom_error::custom_error;

custom_error! {#[derive(Clone)] pub QvOpenApiError
    BadRequestError{ message: String } = "Bad request: {message}",
    WmcaDllLoadingError = "Failed to load wmca.dll",
    WindowCreationError = "Failed to create a window",
    WindowAlreadyCreatedError = "Window already created",
    WindowNotCreatedError = "Window not created",
    WindowUnknownEventError{ wparam: usize } = "Unknown event {wparam}",
    EventUnimplementedError{ event: String } = "Unimplemented event {event}",
    ReturnCodeError{ code: i32 } = "Return code {code}",
    NotConnectedError = "Not connected",
    QvApiMessageError{ message_code: String, message: String } = "[{message_code}] {message}",
    ParseDateTimeError = "Failed to parse datetime",
    AlreadyConnectedError = "Already connected",
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

impl From<chrono::ParseError> for QvOpenApiError {
    fn from(_e: chrono::ParseError) -> Self {
        QvOpenApiError::ParseDateTimeError
    }
}
