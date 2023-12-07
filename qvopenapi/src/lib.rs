extern crate qvopenapi_sys;
extern crate qvopenapi_bindings;
#[macro_use]
extern crate lazy_static;
pub mod error;
pub mod models;
mod window_mgr;
mod wmca_lib;
mod client;
mod utils;

pub use wmca_lib::{init, is_connected, set_server, set_port};
use log::*;
pub use window_mgr::{WindowHelper, WindowStatus};
pub use client::{AbstractQvOpenApiClient, QvOpenApiClientMessageHandler, QvOpenApiClient, QvOpenApiRequest};
