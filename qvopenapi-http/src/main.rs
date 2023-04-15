extern crate qvopenapi;

fn main() -> Result<(), qvopenapi::QvOpenApiError> {
    qvopenapi::init()?;
    println!("is_connected : {}", qvopenapi::is_connected()?);

    Ok(())
}
