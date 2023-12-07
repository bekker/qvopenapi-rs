use chrono::TimeZone;
use qvopenapi_bindings::LoginBlock;
use serde::{Serialize, Deserialize};

use crate::{utils::{from_cp949, SEOUL_TZ}, error::*, wmca_lib, client::QvOpenApiRequest};

pub fn parse_connect(lparam: isize) -> std::result::Result<ConnectResponse, QvOpenApiError> {
    let data_block = lparam as *const LoginBlock;
    unsafe {
        let login_info = (*data_block).login_info;
        let login_datetime_str = from_cp949(&(*login_info).login_datetime); // "20230506203715"
        let server_name = String::from(from_cp949(&(*login_info).server_name).trim()); // "htsi194        "
        let user_id = from_cp949(&(*login_info).user_id);
        let account_count_str = from_cp949(&(*login_info).account_count); // "002"

        let account_count: usize = account_count_str.parse().unwrap();

        let account_infoes = (*login_info)
            .account_infoes
            .iter()
            .take(account_count)
            .map(|account_info_raw| {
                let account_no = from_cp949(&account_info_raw.account_no);
                let account_name = String::from(from_cp949(&account_info_raw.account_name).trim());
                let act_pdt_cdz3 = from_cp949(&account_info_raw.act_pdt_cdz3);
                let amn_tab_cdz4 = from_cp949(&account_info_raw.amn_tab_cdz4);
                let expr_datez8 = String::from(from_cp949(&account_info_raw.expr_datez8).trim());
                let bulk_granted = account_info_raw.granted == 'G' as i8;
                AccountInfoResponse {
                    account_no,
                    account_name,
                    act_pdt_cdz3,
                    amn_tab_cdz4,
                    expr_datez8,
                    bulk_granted,
                }
            })
            .collect();

        let login_timestamp = SEOUL_TZ.datetime_from_str(&login_datetime_str, "%Y%m%d%H%M%S")?.timestamp();

        Ok(ConnectResponse {
            login_timestamp,
            server_name,
            user_id,
            account_count,
            account_infoes,
        })
    }
}

pub const TR_INDEX_CONNECT: i32 = 1;
pub const TR_CODE_CONNECT: &str = "_connect";

#[derive(Debug, Clone, Deserialize)]
pub struct ConnectRequest {
    pub account_type: AccountType,
    pub id: String,
    pub password: String,
    pub cert_password: String,
}

#[derive(Debug, strum_macros::Display, Clone, Copy, Deserialize)]
pub enum AccountType {
    QV,
    NAMUH,
}

impl QvOpenApiRequest for ConnectRequest {
    fn before_post(&self) -> Result<(), QvOpenApiError> {
        if wmca_lib::is_connected()? {
            return Err(QvOpenApiError::AlreadyConnectedError);
        }

        Ok(())
    }

    fn call_lib(&self, _tr_index: i32, hwnd: isize) -> Result<(), QvOpenApiError> {
        wmca_lib::connect(
            hwnd,
            self.account_type,
            &self.id,
            &self.password,
            &self.cert_password,
        )
    }

    fn get_tr_code(&self) -> &str {
        TR_CODE_CONNECT
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ConnectResponse {
    pub login_timestamp: i64,
    pub server_name: String,
    pub user_id: String,
    pub account_count: usize,
    pub account_infoes: Vec<AccountInfoResponse>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AccountInfoResponse {
    pub account_no: String,
    pub account_name: String,
    // 상품 코드
    pub act_pdt_cdz3: String,
    // 관리점 코드
    pub amn_tab_cdz4: String,
    // 위임 만기일
    pub expr_datez8: String,
    // 일괄주문 허용계좌(G:허용)
    pub bulk_granted: bool,
}
