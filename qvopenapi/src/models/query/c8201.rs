use std::ffi::c_char;
use std::mem::size_of;

use serde::Serialize;
use serde_json::{Value, json};

use crate::utils::from_cp949;
use crate::{QueryRequest, QvOpenApiError};
use qvopenapi_bindings::{Tc8201InBlock, Tc8201OutBlock, Tc8201OutBlock1};

pub const TR_CODE_C8201: &str = "c8201";

pub fn make_c8201_request(
	tr_index: i32,
	account_index: i32,
	balance_type: char,
) -> Result<QueryRequest<Tc8201InBlock>, QvOpenApiError> {
	let req = QueryRequest {
		tr_index,
		tr_code: TR_CODE_C8201,
		input: Box::new(Tc8201InBlock {
			pswd_noz44: [' ' as c_char; 44],
			_pswd_noz44: ' ' as c_char,
			bnc_bse_cdz1: [balance_type as c_char],
			_bnc_bse_cdz1: ' ' as c_char,
		}),
		account_index,
	};
	return Ok(req);
}

pub fn parse_c8201_response(block_data: *const c_char, _block_len: i32) -> Result<Value, QvOpenApiError> {
	unsafe {
		let res = block_data as *const Tc8201OutBlock;
		Ok(json!(C8201Response {
			dpsit_amtz16: from_cp949(&(*res).dpsit_amtz16).parse().unwrap(),
			mrgn_amtz16: from_cp949(&(*res).dpsit_amtz16).parse().unwrap(),
			mgint_npaid_amtz16: from_cp949(&(*res).mgint_npaid_amtz16).parse().unwrap(),
			chgm_pos_amtz16: from_cp949(&(*res).chgm_pos_amtz16).parse().unwrap(),
			cash_mrgn_amtz16: from_cp949(&(*res).cash_mrgn_amtz16).parse().unwrap(),
			subst_mgamt_amtz16: from_cp949(&(*res).subst_mgamt_amtz16).parse().unwrap(),
			coltr_ratez6: from_cp949(&(*res).coltr_ratez6).parse().unwrap(),
			rcble_amtz16: from_cp949(&(*res).rcble_amtz16).parse().unwrap(),
			order_pos_csamtz16: from_cp949(&(*res).order_pos_csamtz16).parse().unwrap(),
			ecn_pos_csamtz16: from_cp949(&(*res).ecn_pos_csamtz16).parse().unwrap(),
			nordm_loan_amtz16: from_cp949(&(*res).nordm_loan_amtz16).parse().unwrap(),
			etc_lend_amtz16: from_cp949(&(*res).etc_lend_amtz16).parse().unwrap(),
			subst_amtz16: from_cp949(&(*res).subst_amtz16).parse().unwrap(),
			sln_sale_amtz16: from_cp949(&(*res).sln_sale_amtz16).parse().unwrap(),
			bal_buy_ttamtz16: from_cp949(&(*res).bal_buy_ttamtz16).parse().unwrap(),
			bal_ass_ttamtz16: from_cp949(&(*res).bal_ass_ttamtz16).parse().unwrap(),
			asset_tot_amtz16: from_cp949(&(*res).asset_tot_amtz16).parse().unwrap(),
			actvt_type10: from_cp949(&(*res).actvt_type10).parse().unwrap(),
			lend_amtz16: from_cp949(&(*res).lend_amtz16).parse().unwrap(),
			accnt_mgamt_ratez6: from_cp949(&(*res).accnt_mgamt_ratez6).parse().unwrap(),
			sl_mrgn_amtz16: from_cp949(&(*res).sl_mrgn_amtz16).parse().unwrap(),
			pos_csamt1z16: from_cp949(&(*res).pos_csamt1z16).parse().unwrap(),
			pos_csamt2z16: from_cp949(&(*res).pos_csamt2z16).parse().unwrap(),
			pos_csamt3z16: from_cp949(&(*res).pos_csamt3z16).parse().unwrap(),
			pos_csamt4z16: from_cp949(&(*res).pos_csamt4z16).parse().unwrap(),
			dpsit_amtz_d1_16: from_cp949(&(*res).dpsit_amtz_d1_16).parse().unwrap(),
			dpsit_amtz_d2_16: from_cp949(&(*res).dpsit_amtz_d2_16).parse().unwrap(),
			noticez30: from_cp949(&(*res).noticez30).parse().unwrap(),
			tot_eal_plsz18: from_cp949(&(*res).tot_eal_plsz18).parse().unwrap(),
			pft_rtz15: from_cp949(&(*res).pft_rtz15).parse().unwrap(),
 		}))
	}
}

pub fn parse_c8201_response1(block_data: *const c_char, block_len: i32) -> Result<Value, QvOpenApiError> {
	unsafe {
		let block_count = block_len as usize / size_of::<Tc8201OutBlock1>();
		let res: &[Tc8201OutBlock1] = core::slice::from_raw_parts(block_data as *const Tc8201OutBlock1, block_count);

        let ret: Vec<Result<C8201Response1, QvOpenApiError>> = res.iter()
            .map(|res| {
                Ok(C8201Response1 {
					issue_codez6: from_cp949(&(*res).issue_codez6).parse().unwrap(),
					issue_namez40: from_cp949(&(*res).issue_namez40).trim().into(),
					bal_typez6: from_cp949(&(*res).bal_typez6).parse().unwrap(),
					loan_datez10: from_cp949(&(*res).loan_datez10).parse().unwrap(),
					bal_qtyz16: from_cp949(&(*res).bal_qtyz16).trim().parse().unwrap_or(0),
					unstl_qtyz16: from_cp949(&(*res).unstl_qtyz16).parse().unwrap_or(0),
					slby_amtz16: from_cp949(&(*res).slby_amtz16).parse().unwrap(),
					prsnt_pricez16: from_cp949(&(*res).prsnt_pricez16).parse().unwrap(),
					lsnpf_amtz16: from_cp949(&(*res).lsnpf_amtz16).parse().unwrap(),
					earn_ratez9: from_cp949(&(*res).earn_ratez9).parse().unwrap(),
					mrgn_codez4: from_cp949(&(*res).mrgn_codez4).parse().unwrap(),
					jan_qtyz16: from_cp949(&(*res).jan_qtyz16).parse().unwrap(),
					expr_datez10: from_cp949(&(*res).expr_datez10).parse().unwrap(),
					ass_amtz16: from_cp949(&(*res).ass_amtz16).parse().unwrap(),
					issue_mgamt_ratez6: from_cp949(&(*res).issue_mgamt_ratez6).parse().unwrap(),
					medo_slby_amtz16: from_cp949(&(*res).medo_slby_amtz16).parse().unwrap(),
					post_lsnpf_amtz16: from_cp949(&(*res).post_lsnpf_amtz16).parse().unwrap(),
				 })
            })
            .collect();

		let results: Result<Vec<C8201Response1>, QvOpenApiError> = ret
			.into_iter()
			.collect();
		Ok(json!(C8201Response1Vec {
			results: results?
		}))
	}
}

#[derive(Debug, Clone, Serialize)]
struct C8201Response {
	pub dpsit_amtz16: i64, //예수금
    pub mrgn_amtz16: i64, //신용융자금
    pub mgint_npaid_amtz16: i64, //이자미납금
    pub chgm_pos_amtz16: i64, //출금가능금액
    pub cash_mrgn_amtz16: i64, //현금증거금
    pub subst_mgamt_amtz16: i64, //대용증거금
    pub coltr_ratez6: String, //담보비율
    pub rcble_amtz16: i64, //현금미수금
    pub order_pos_csamtz16: i64, //주문가능액
    pub ecn_pos_csamtz16: i64, //ECN주문가능액
    pub nordm_loan_amtz16: i64, //미상환금
    pub etc_lend_amtz16: i64, //기타대여금
    pub subst_amtz16: i64, //대용금액
    pub sln_sale_amtz16: i64, //대주담보금
    pub bal_buy_ttamtz16: i64, //매입원가(계좌합산)
    pub bal_ass_ttamtz16: i64, //평가금액(계좌합산)
    pub asset_tot_amtz16: i64, //순자산액(계좌합산)
    pub actvt_type10: String, //활동유형
    pub lend_amtz16: i64, //대출금
    pub accnt_mgamt_ratez6: String, //계좌증거금율
    pub sl_mrgn_amtz16: i64, //매도증거금
    pub pos_csamt1z16: i64, //20%주문가능금액
    pub pos_csamt2z16: i64, //30%주문가능금액
    pub pos_csamt3z16: i64, //40%주문가능금액
    pub pos_csamt4z16: i64, //100%주문가능금액
    pub dpsit_amtz_d1_16: i64, //D1예수금
    pub dpsit_amtz_d2_16: i64, //D2예수금
    pub noticez30: String, //공지사항             /*To-be에없음*/
    pub tot_eal_plsz18: String, //총평가손익
    pub pft_rtz15: String, //수익율
}

#[derive(Debug, Clone, Serialize)]
struct C8201Response1Vec {
	pub results: Vec<C8201Response1>
}

#[derive(Debug, Clone, Serialize)]
struct C8201Response1 {
	pub issue_codez6: String, //종목번호
    pub issue_namez40: String, //종목명
    pub bal_typez6: String, //잔고유형
    pub loan_datez10: String, //대출일
    pub bal_qtyz16: i64, //잔고수량
    pub unstl_qtyz16: i64, //미결제량
    pub slby_amtz16: String, //평균매입가
    pub prsnt_pricez16: String, //현재가
    pub lsnpf_amtz16: String, //손익(천원)
    pub earn_ratez9: String, //손익율
    pub mrgn_codez4: String, //신용유형
    pub jan_qtyz16: String, //잔량
    pub expr_datez10: String, //만기일
    pub ass_amtz16: String, //평가금액
    pub issue_mgamt_ratez6: String, //종목증거금율         /*float->char*/
    pub medo_slby_amtz16: String, //평균매도가
    pub post_lsnpf_amtz16: String, //매도손익
}

pub const BLOCK_NAME_C8201_OUT: &str = "c8201OutBlock";
pub const BLOCK_NAME_C8201_OUT1_VEC: &str = "c8201OutBlock1";
