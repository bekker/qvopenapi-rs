use std::sync::Arc;

use qvopenapi::{SimpleQvOpenApiClient, QvOpenApiClientMessageHandler};

pub struct FutureQvOpenApiClient {
    inner: SimpleQvOpenApiClient,
}

impl qvopenapi::QvOpenApiClient for FutureQvOpenApiClient {
    fn get_handler(&self) -> Arc<QvOpenApiClientMessageHandler> {
        self.inner.get_handler()
    }
}
