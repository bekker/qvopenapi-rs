use log::*;
use std::time::Duration;

extern crate qvopenapi;
use qvopenapi::AccountType;

fn do_run() -> Result<(), qvopenapi::QvOpenApiError> {
    env_logger::init_from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "debug"),
    );
    qvopenapi::init()?;
    info!("is_connected : {}", qvopenapi::is_connected()?);
    std::thread::sleep(Duration::from_millis(100));
    qvopenapi::connect(AccountType::NAMUH, "id", "password", "cert_pw")?;
    info!("is_connected : {}", qvopenapi::is_connected()?);
    std::thread::sleep(Duration::from_millis(100));
    qvopenapi::connect(AccountType::NAMUH, "id2", "password", "cert_pw")?;
    qvopenapi::query("c1101", "asdf", 0)?;
    std::thread::sleep(Duration::from_millis(5000));

    Ok(())
}

fn main() {
    match do_run() {
        Ok(_) => {}
        Err(e) => {
            error!("Error occured: {}", e);
        }
    }
}
