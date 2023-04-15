use std::time::Duration;

extern crate qvopenapi;

fn main() -> Result<(), qvopenapi::QvOpenApiError> {
    qvopenapi::init()?;
    println!("is_connected : {}", qvopenapi::is_connected()?);
    std::thread::sleep(Duration::from_millis(5000));

    Ok(())
}
