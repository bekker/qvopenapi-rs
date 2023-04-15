extern crate qvopenapi;

fn main() -> Result<(), qvopenapi::QvOpenApiError> {
    println!("Loading wmca.dll");
    qvopenapi::load_lib()?;
    println!("Loaded wmca.dll");
    println!("is_connected : {}", qvopenapi::is_connected()?);

    Ok(())
}
