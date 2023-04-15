extern crate winapi;
extern crate qvopenapi_sys;

pub fn init() {
    println!("Hello from an example! {}", qvopenapi_sys::add(1, 2));
}
