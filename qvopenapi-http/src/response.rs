use serde::Serialize;

#[derive(Serialize)]
pub struct HttpMessageResponse {
    pub message: String,
}
