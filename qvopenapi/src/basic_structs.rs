use std::ffi::{c_char, c_int};

#[repr(C)]
pub struct OutDataBlock<T> {
    pub tr_index: c_int,
    pub p_data: *const ReceivedData<T>,
}

#[repr(C)]
pub struct ReceivedData<T> {
    pub block_name: *const c_char,
    pub sz_data: *const T,
    pub len: c_int,
}

#[repr(C)]
pub struct MessageHeader {
    pub message_code: [c_char; 5],
    pub message: [c_char; 80],
}

#[repr(C)]
pub struct LoginBlock {
    pub tr_index: c_int,
    pub login_info: *const LoginInfo,
}

#[repr(C)]
pub struct LoginInfo {
    pub login_datetime: [c_char; 14],
    pub server_name: [c_char; 15],
    pub user_id: [c_char; 8],
    pub account_count: [c_char; 3],
    pub account_infoes: [AccountInfo; 999],
}

#[repr(C)]
pub struct AccountInfo {
    pub account_no: [c_char; 11],
    pub account_name: [c_char; 40],
    // 상품 코드
    pub act_pdt_cdz3: [c_char; 3],
    // 관리점 코드
    pub amn_tab_cdz4: [c_char; 4],
    // 위임 만기일
    pub expr_datez8: [c_char; 8],
    // 일괄주문 허용계좌(G:허용)
    pub granted: c_char,
    // filler
    pub filler: [c_char; 189],
}
