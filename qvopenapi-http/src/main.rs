use log::*;
use std::time::Duration;

extern crate qvopenapi;
use qvopenapi::AccountType;

async fn do_run() -> Result<(), qvopenapi::QvOpenApiError> {
    env_logger::init_from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "debug"),
    );
    qvopenapi::init()?;
    info!("is_connected : {}", qvopenapi::is_connected()?);
    qvopenapi::connect(AccountType::NAMUH, "abcd", "abcd", "abcd").await;
    info!("is_connected : {}", qvopenapi::is_connected()?);
    qvopenapi::query("c1101", "asdf", 0).await;
    std::thread::sleep(Duration::from_millis(100));
    //qvopenapi::connect(AccountType::NAMUH, "id2", "password", "cert_pw")?;
    std::thread::sleep(Duration::from_millis(50000));

    Ok(())
}

#[tokio::main]
async fn main() {
    match do_run().await {
        Ok(_) => {}
        Err(e) => {
            error!("Error occured: {}", e);
        }
    }
}
