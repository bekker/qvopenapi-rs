extern crate qvopenapi;
#[macro_use]
extern crate lazy_static;

mod client;
mod future;
mod query;

pub use client::FutureQvOpenApiClient;
