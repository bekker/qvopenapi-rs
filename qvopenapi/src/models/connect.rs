use chrono::{DateTime, FixedOffset, TimeZone};
use qvopenapi_bindings::LoginBlock;

use crate::{utils::{from_cp949, SEOUL_TZ}, QvOpenApiError};

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

        let login_datetime = SEOUL_TZ.datetime_from_str(&login_datetime_str, "%Y%m%d%H%M%S")?;

        Ok(ConnectResponse {
            login_datetime,
            server_name,
            user_id,
            account_count,
            account_infoes,
        })
    }
}

pub struct ConnectResponse {
    pub login_datetime: DateTime<FixedOffset>,
    pub server_name: String,
    pub user_id: String,
    pub account_count: usize,
    pub account_infoes: Vec<AccountInfoResponse>,
}

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
