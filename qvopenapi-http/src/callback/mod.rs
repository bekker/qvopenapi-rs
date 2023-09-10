use ::log::*;
use qvopenapi::{QvOpenApiClient, AbstractQvOpenApiClient};

pub fn setup_callbacks(client: &mut QvOpenApiClient) {
    client.on_connect(Box::new(|res| {
        info!("Connected: account count {}", res.account_count);
    }));
    client.on_data(Box::new(|res| {
        match res.block_name.as_str() {
            qvopenapi::BLOCK_NAME_C8201_OUT => {
                let data = &res.block_data;
                info!("출금가능금액: {}", data["chgm_pos_amtz16"])
            }
            qvopenapi::BLOCK_NAME_C8201_OUT1_ARRAY => {
                for data in res.block_data.as_array().unwrap().iter() {
                    info!("종목번호: {}, 종목명: {}, 보유수: {}", data["issue_codez6"], data["issue_namez40"], data["bal_qtyz16"])
                }
            }
            _ => {
                error!("Unknown block name {}", res.block_name)
            }
        }
    }));
}
