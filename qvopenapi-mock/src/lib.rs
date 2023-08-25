extern crate qvopenapi;
#[macro_use]
extern crate lazy_static;

use std::{sync::Arc, thread::sleep, time::Duration};

use chrono::FixedOffset;
use qvopenapi::*;

lazy_static! {
    static ref SEOUL_TZ: FixedOffset = FixedOffset::east_opt(9 * 3600).unwrap();
}

pub struct MockQvOpenApiClient {
    handler: Arc<QvOpenApiClientMessageHandler>,
}

impl MockQvOpenApiClient {
    pub fn new() -> MockQvOpenApiClient {
        MockQvOpenApiClient {
            handler: Arc::new(QvOpenApiClientMessageHandler::new()),
        }
    }
}

impl QvOpenApiClientEventHandleable for MockQvOpenApiClient {
    fn get_handler(&self) -> Arc<QvOpenApiClientMessageHandler> {
        self.handler.clone()
    }
}

impl AbstractQvOpenApiClient for MockQvOpenApiClient {
    fn set_hwnd(&self, _new_hwnd: isize) {
        // do nothing
    }

    fn query(&self, req: Arc<dyn QvOpenApiRequest>) -> Result<(), QvOpenApiError> {
        let handler: Arc<QvOpenApiClientMessageHandler> = self.get_handler().clone();
        match req.get_tr_code() {
            TR_CODE_CONNECT => {
                std::thread::spawn(move || {
                    sleep(Duration::from_millis(100));
                    let callbacks = handler.message_handler.read().unwrap();
                    (callbacks.on_connect)(Arc::new(make_connect_response()))
                });
                Ok(())
            },
            TR_CODE_C8201 => {
                std::thread::spawn(move || {
                    sleep(Duration::from_millis(100));
                    let callbacks = handler.message_handler.read().unwrap();
                    (callbacks.on_data)(&DataResponse {
                        tr_index: req.get_tr_index(),
                        block_name: BLOCK_NAME_C8201_OUT1_VEC.into(),
                        block_len: 0,
                        block_data: Box::new(make_c8201_response1()),
                    });
                    sleep(Duration::from_millis(10));
                    (callbacks.on_data)(&DataResponse {
                        tr_index: req.get_tr_index(),
                        block_name: BLOCK_NAME_C8201_OUT.into(),
                        block_len: 0,
                        block_data: Box::new(make_c8201_response()),
                    })
                });
                Ok(())
            }
            _ => Err(QvOpenApiError::BadRequestError { message: String::from("Not supported mock request") })
        }
    }
}

fn make_connect_response() -> ConnectResponse {
    let account_infoes = vec![
        AccountInfoResponse {
            account_no: "1234567".into(),
            account_name: "accountname".into(),
            act_pdt_cdz3: "".into(),
            amn_tab_cdz4: "".into(),
            expr_datez8: "20230505".into(),
            bulk_granted: false,
        }
    ];

    ConnectResponse {
        login_datetime: chrono::Utc::now().with_timezone(&SEOUL_TZ),
        server_name: "htsd49".into(),
        user_id: "mockid".into(),
        account_count: account_infoes.len(),
        account_infoes,
    }
}

pub fn make_c8201_response() -> C8201Response {
    C8201Response {
        dpsit_amtz16: 0,
        mrgn_amtz16: 0,
        mgint_npaid_amtz16: 0,
        chgm_pos_amtz16: 0,
        cash_mrgn_amtz16: 0,
        subst_mgamt_amtz16: 0,
        coltr_ratez6: "".into(),
        rcble_amtz16: 0,
        order_pos_csamtz16: 0,
        ecn_pos_csamtz16: 0,
        nordm_loan_amtz16: 0,
        etc_lend_amtz16: 0,
        subst_amtz16: 0,
        sln_sale_amtz16: 0,
        bal_buy_ttamtz16: 0,
        bal_ass_ttamtz16: 0,
        asset_tot_amtz16: 0,
        actvt_type10: "".into(),
        lend_amtz16: 0,
        accnt_mgamt_ratez6: "".into(),
        sl_mrgn_amtz16: 0,
        pos_csamt1z16: 0,
        pos_csamt2z16: 0,
        pos_csamt3z16: 0,
        pos_csamt4z16: 0,
        dpsit_amtz_d1_16: 0,
        dpsit_amtz_d2_16: 0,
        noticez30: "".into(),
        tot_eal_plsz18: "".into(),
        pft_rtz15: "".into(),
    }
}

pub fn make_c8201_response1() -> Vec<C8201Response1> {
    vec![
        C8201Response1 {
            issue_codez6: "035420".into(),
            issue_namez40: "NAVER".into(),
            bal_typez6: "".into(),
            loan_datez10: "".into(),
            bal_qtyz16: 20,
            unstl_qtyz16: 0,
            slby_amtz16: "".into(),
            prsnt_pricez16: "".into(),
            lsnpf_amtz16: "".into(),
            earn_ratez9: "".into(),
            mrgn_codez4: "".into(),
            jan_qtyz16: "".into(),
            expr_datez10: "".into(),
            ass_amtz16: "".into(),
            issue_mgamt_ratez6: "".into(),
            medo_slby_amtz16: "".into(),
            post_lsnpf_amtz16: "".into(),
        },
        C8201Response1 {
            issue_codez6: "294400".into(),
            issue_namez40: "KOSEF 200TR".into(),
            bal_typez6: "".into(),
            loan_datez10: "".into(),
            bal_qtyz16: 338,
            unstl_qtyz16: 0,
            slby_amtz16: "".into(),
            prsnt_pricez16: "".into(),
            lsnpf_amtz16: "".into(),
            earn_ratez9: "".into(),
            mrgn_codez4: "".into(),
            jan_qtyz16: "".into(),
            expr_datez10: "".into(),
            ass_amtz16: "".into(),
            issue_mgamt_ratez6: "".into(),
            medo_slby_amtz16: "".into(),
            post_lsnpf_amtz16: "".into(),
        }
    ]
}
