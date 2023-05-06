use std::time::Instant;

pub struct ConnectResponse {
    pub login_datetime: Instant,
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

pub struct QueryResponse {
}
