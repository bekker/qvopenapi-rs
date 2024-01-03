use std::ffi::c_char;
use std::mem::size_of;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::utils::{parse_number, parse_ratio, parse_ratio_str, parse_string};
use crate::{error::*, models::*};
use qvopenapi_bindings::{Tc8201InBlock, Tc8201OutBlock, Tc8201OutBlock1};

pub const TR_CODE_C8201: &str = "c8201";

#[derive(Debug, Clone, Deserialize)]
pub struct C8201Request {
    pub account_index: i32,
    pub balance_type: char,
}

impl C8201Request {
    pub fn new(account_index: i32, balance_type: char) -> C8201Request {
        C8201Request {
            account_index,
            balance_type,
        }
    }

    pub fn into_raw(&self) -> Arc<RawQueryRequest<Tc8201InBlock>> {
        Arc::new(RawQueryRequest::new(
            TR_CODE_C8201,
            self.account_index,
            Box::new(Tc8201InBlock {
                pswd_noz44: [' ' as c_char; 44],
                _pswd_noz44: ' ' as c_char,
                bnc_bse_cdz1: [self.balance_type as c_char],
                _bnc_bse_cdz1: ' ' as c_char,
            }),
        ))
    }
}

pub fn parse_c8201_response(
    block_data: *const c_char,
    _block_len: i32,
) -> Result<Value, QvOpenApiError> {
    unsafe {
        let res = &(*(block_data as *const Tc8201OutBlock));
        Ok(json!(C8201Response {
            dpsit_amtz16: parse_number(&res.dpsit_amtz16)?,
            mrgn_amtz16: parse_number(&res.dpsit_amtz16)?,
            mgint_npaid_amtz16: parse_number(&res.mgint_npaid_amtz16)?,
            chgm_pos_amtz16: parse_number(&res.chgm_pos_amtz16)?,
            cash_mrgn_amtz16: parse_number(&res.cash_mrgn_amtz16)?,
            subst_mgamt_amtz16: parse_number(&res.subst_mgamt_amtz16)?,
            coltr_ratez6: parse_string(&res.coltr_ratez6)?,
            rcble_amtz16: parse_number(&res.rcble_amtz16)?,
            order_pos_csamtz16: parse_number(&res.order_pos_csamtz16)?,
            ecn_pos_csamtz16: parse_number(&res.ecn_pos_csamtz16)?,
            nordm_loan_amtz16: parse_number(&res.nordm_loan_amtz16)?,
            etc_lend_amtz16: parse_number(&res.etc_lend_amtz16)?,
            subst_amtz16: parse_number(&res.subst_amtz16)?,
            sln_sale_amtz16: parse_number(&res.sln_sale_amtz16)?,
            bal_buy_ttamtz16: parse_number(&res.bal_buy_ttamtz16)?,
            bal_ass_ttamtz16: parse_number(&res.bal_ass_ttamtz16)?,
            asset_tot_amtz16: parse_number(&res.asset_tot_amtz16)?,
            actvt_type10: parse_string(&res.actvt_type10)?,
            lend_amtz16: parse_number(&res.lend_amtz16)?,
            accnt_mgamt_ratez6: parse_string(&res.accnt_mgamt_ratez6)?,
            sl_mrgn_amtz16: parse_number(&res.sl_mrgn_amtz16)?,
            pos_csamt1z16: parse_number(&res.pos_csamt1z16)?,
            pos_csamt2z16: parse_number(&res.pos_csamt2z16)?,
            pos_csamt3z16: parse_number(&res.pos_csamt3z16)?,
            pos_csamt4z16: parse_number(&res.pos_csamt4z16)?,
            dpsit_amtz_d1_16: parse_number(&res.dpsit_amtz_d1_16)?,
            dpsit_amtz_d2_16: parse_number(&res.dpsit_amtz_d2_16)?,
            noticez30: parse_string(&res.noticez30)?,
            tot_eal_plsz18: parse_string(&res.tot_eal_plsz18)?,
            pft_rtz15: parse_ratio(&res.pft_rtz15)?,
        }))
    }
}

pub fn parse_c8201_response1_array(
    block_data: *const c_char,
    block_len: i32,
) -> Result<Value, QvOpenApiError> {
    unsafe {
        let block_count = block_len as usize / size_of::<Tc8201OutBlock1>();
        let res: &[Tc8201OutBlock1] =
            core::slice::from_raw_parts(block_data as *const Tc8201OutBlock1, block_count);

        let ret: Result<Vec<C8201Response1>, QvOpenApiError> =
            res.iter().map(parse_c8201_response1).collect();
        Ok(json!(ret?))
    }
}

fn parse_c8201_response1(res: &Tc8201OutBlock1) -> Result<C8201Response1, QvOpenApiError> {
    Ok(C8201Response1 {
        issue_codez6: parse_string(&res.issue_codez6)?,
        issue_namez40: parse_string(&res.issue_namez40)?,
        bal_typez6: parse_string(&res.bal_typez6)?,
        loan_datez10: parse_string(&res.loan_datez10)?,
        bal_qtyz16: parse_number(&res.bal_qtyz16)?,
        unstl_qtyz16: parse_number(&res.unstl_qtyz16)?,
        slby_amtz16: parse_number(&res.slby_amtz16)?,
        prsnt_pricez16: parse_number(&res.prsnt_pricez16)?,
        lsnpf_amtz16: parse_number(&res.lsnpf_amtz16)?,
        earn_ratez9: parse_ratio(&res.earn_ratez9)?,
        mrgn_codez4: parse_string(&res.mrgn_codez4)?,
        jan_qtyz16: parse_number(&res.jan_qtyz16)?,
        expr_datez10: parse_string(&res.expr_datez10)?,
        ass_amtz16: parse_number(&res.ass_amtz16)?,
        issue_mgamt_ratez6: parse_ratio_str(&res.issue_mgamt_ratez6)?,
        medo_slby_amtz16: parse_number(&res.medo_slby_amtz16)?,
        post_lsnpf_amtz16: parse_number(&res.post_lsnpf_amtz16)?,
    })
}

#[derive(Debug, Clone, Serialize)]
struct C8201Response {
    pub dpsit_amtz16: Option<i64>,       //예수금
    pub mrgn_amtz16: Option<i64>,        //신용융자금
    pub mgint_npaid_amtz16: Option<i64>, //이자미납금
    pub chgm_pos_amtz16: Option<i64>,    //출금가능금액
    pub cash_mrgn_amtz16: Option<i64>,   //현금증거금
    pub subst_mgamt_amtz16: Option<i64>, //대용증거금
    pub coltr_ratez6: String,            //담보비율
    pub rcble_amtz16: Option<i64>,       //현금미수금
    pub order_pos_csamtz16: Option<i64>, //주문가능액
    pub ecn_pos_csamtz16: Option<i64>,   //ECN주문가능액
    pub nordm_loan_amtz16: Option<i64>,  //미상환금
    pub etc_lend_amtz16: Option<i64>,    //기타대여금
    pub subst_amtz16: Option<i64>,       //대용금액
    pub sln_sale_amtz16: Option<i64>,    //대주담보금
    pub bal_buy_ttamtz16: Option<i64>,   //매입원가(계좌합산)
    pub bal_ass_ttamtz16: Option<i64>,   //평가금액(계좌합산)
    pub asset_tot_amtz16: Option<i64>,   //순자산액(계좌합산)
    pub actvt_type10: String,            //활동유형
    pub lend_amtz16: Option<i64>,        //대출금
    pub accnt_mgamt_ratez6: String,      //계좌증거금율
    pub sl_mrgn_amtz16: Option<i64>,     //매도증거금
    pub pos_csamt1z16: Option<i64>,      //20%주문가능금액
    pub pos_csamt2z16: Option<i64>,      //30%주문가능금액
    pub pos_csamt3z16: Option<i64>,      //40%주문가능금액
    pub pos_csamt4z16: Option<i64>,      //100%주문가능금액
    pub dpsit_amtz_d1_16: Option<i64>,   //D1예수금
    pub dpsit_amtz_d2_16: Option<i64>,   //D2예수금
    pub noticez30: String,               //공지사항             /*To-be에없음*/
    pub tot_eal_plsz18: String,          //총평가손익
    pub pft_rtz15: Option<f64>,          //수익율
}

#[derive(Debug, Clone, Serialize)]
struct C8201Response1 {
    pub issue_codez6: String,            //종목번호
    pub issue_namez40: String,           //종목명
    pub bal_typez6: String,              //잔고유형
    pub loan_datez10: String,            //대출일
    pub bal_qtyz16: Option<i64>,         //잔고수량
    pub unstl_qtyz16: Option<i64>,       //미결제량
    pub slby_amtz16: Option<i64>,        //평균매입가
    pub prsnt_pricez16: Option<i64>,     //현재가
    pub lsnpf_amtz16: Option<i64>,       //손익(천원)
    pub earn_ratez9: Option<f64>,        //손익율
    pub mrgn_codez4: String,             //신용유형
    pub jan_qtyz16: Option<i64>,         //잔량
    pub expr_datez10: String,            //만기일
    pub ass_amtz16: Option<i64>,         //평가금액
    pub issue_mgamt_ratez6: Option<f64>, //종목증거금율         /*float->char*/
    pub medo_slby_amtz16: Option<i64>,   //평균매도가
    pub post_lsnpf_amtz16: Option<i64>,  //매도손익
}

pub const BLOCK_NAME_C8201_OUT: &str = "c8201OutBlock";
pub const BLOCK_NAME_C8201_OUT1_ARRAY: &str = "c8201OutBlock1";
