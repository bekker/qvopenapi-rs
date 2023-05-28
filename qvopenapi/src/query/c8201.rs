use std::ffi::c_char;

use crate::{QueryRequest, wmca_lib, QvOpenApiError};

pub const TR_CODE_C8201: &str = "c8201";

pub fn make_c8201_request(
	tr_index: i32,
	account_index: i32,
	password: &str,
	balance_type: char,
) -> Result<QueryRequest<Tc8201InBlock>, QvOpenApiError> {
	let mut req = QueryRequest {
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
	wmca_lib::set_account_index_pwd(&mut req.input.pswd_noz44, account_index, password)?;
	return Ok(req);
}

#[repr(C)]
pub struct Tc8201InBlock {
    pub pswd_noz44: [c_char; 44],
    pub _pswd_noz44: c_char, //비밀번호
    pub bnc_bse_cdz1: [c_char; 1],
    pub _bnc_bse_cdz1: c_char, //잔고구분
}

pub struct C8201Response {
	pub withdrawable: usize, // 출금가능금액
}

pub struct C8201Response1 {
	pub symbol: String, // 종목번호
	pub name: String, // 종목명
	pub quantity: usize, // 현재 잔고
	pub present_price: isize, // 현재가
	pub average_buy_price: isize, // 평균 매입가
	pub earning_k: isize, // 손익 (천원)
}

const BLOCK_NAME_C8201_OUT: &str = "c8201OutBlock";
#[repr(C)]
struct Tc8201OutBlock {
    pub dpsit_amtz16: [c_char; 16],
    pub _dpsit_amtz16: c_char, //예수금
    pub mrgn_amtz16: [c_char; 16],
    pub _mrgn_amtz16: c_char, //신용융자금
    pub mgint_npaid_amtz16: [c_char; 16],
    pub _mgint_npaid_amtz16: c_char, //이자미납금
    pub chgm_pos_amtz16: [c_char; 16],
    pub _chgm_pos_amtz16: c_char, //출금가능금액
    pub cash_mrgn_amtz16: [c_char; 16],
    pub _cash_mrgn_amtz16: c_char, //현금증거금
    pub subst_mgamt_amtz16: [c_char; 16],
    pub _subst_mgamt_amtz16: c_char, //대용증거금
    pub coltr_ratez6: [c_char; 6],
    pub _coltr_ratez6: c_char, //담보비율
    pub rcble_amtz16: [c_char; 16],
    pub _rcble_amtz16: c_char, //현금미수금
    pub order_pos_csamtz16: [c_char; 16],
    pub _order_pos_csamtz16: c_char, //주문가능액
    pub ecn_pos_csamtz16: [c_char; 16],
    pub _ecn_pos_csamtz16: c_char, //ECN주문가능액
    pub nordm_loan_amtz16: [c_char; 16],
    pub _nordm_loan_amtz16: c_char, //미상환금
    pub etc_lend_amtz16: [c_char; 16],
    pub _etc_lend_amtz16: c_char, //기타대여금
    pub subst_amtz16: [c_char; 16],
    pub _subst_amtz16: c_char, //대용금액
    pub sln_sale_amtz16: [c_char; 16],
    pub _sln_sale_amtz16: c_char, //대주담보금
    pub bal_buy_ttamtz16: [c_char; 16],
    pub _bal_buy_ttamtz16: c_char, //매입원가(계좌합산)
    pub bal_ass_ttamtz16: [c_char; 16],
    pub _bal_ass_ttamtz16: c_char, //평가금액(계좌합산)
    pub asset_tot_amtz16: [c_char; 16],
    pub _asset_tot_amtz16: c_char, //순자산액(계좌합산)
    pub actvt_type10: [c_char; 10],
    pub _actvt_type10: c_char, //활동유형
    pub lend_amtz16: [c_char; 16],
    pub _lend_amtz16: c_char, //대출금
    pub accnt_mgamt_ratez6: [c_char; 6],
    pub _accnt_mgamt_ratez6: c_char, //계좌증거금율
    pub sl_mrgn_amtz16: [c_char; 16],
    pub _sl_mrgn_amtz16: c_char, //매도증거금
    pub pos_csamt1z16: [c_char; 16],
    pub _pos_csamt1z16: c_char, //20%주문가능금액
    pub pos_csamt2z16: [c_char; 16],
    pub _pos_csamt2z16: c_char, //30%주문가능금액
    pub pos_csamt3z16: [c_char; 16],
    pub _pos_csamt3z16: c_char, //40%주문가능금액
    pub pos_csamt4z16: [c_char; 16],
    pub _pos_csamt4z16: c_char, //100%주문가능금액
    pub dpsit_amtz_d1_16: [c_char; 16],
    pub _dpsit_amtz_d1_16: c_char, //D1예수금
    pub dpsit_amtz_d2_16: [c_char; 16],
    pub _dpsit_amtz_d2_16: c_char, //D2예수금
    pub noticez30: [c_char; 30],
    pub _noticez30: c_char, //공지사항             /*To-be에없음*/
    pub tot_eal_plsz18: [c_char; 18],
    pub _tot_eal_plsz18: c_char, //총평가손익
    pub pft_rtz15: [c_char; 15],
    pub _pft_rtz15: c_char, //수익율
}

const BLOCK_NAME_C8201_OUT1: &str = "c8201OutBlock1";
#[repr(C)]
struct Tc8201OutBlock1 {
    pub issue_codez6: [c_char; 6],
    pub _issue_codez6: c_char, //종목번호
    pub issue_namez40: [c_char; 40],
    pub _issue_namez40: c_char, //종목명
    pub bal_typez6: [c_char; 6],
    pub _bal_typez6: c_char, //잔고유형
    pub loan_datez10: [c_char; 10],
    pub _loan_datez10: c_char, //대출일
    pub bal_qtyz16: [c_char; 16],
    pub _bal_qtyz16: c_char, //잔고수량
    pub unstl_qtyz16: [c_char; 16],
    pub _unstl_qtyz16: c_char, //미결제량
    pub slby_amtz16: [c_char; 16],
    pub _slby_amtz16: c_char, //평균매입가
    pub prsnt_pricez16: [c_char; 16],
    pub _prsnt_pricez16: c_char, //현재가
    pub lsnpf_amtz16: [c_char; 16],
    pub _lsnpf_amtz16: c_char, //손익(천원)
    pub earn_ratez9: [c_char; 9],
    pub _earn_ratez9: c_char, //손익율
    pub mrgn_codez4: [c_char; 4],
    pub _mrgn_codez4: c_char, //신용유형
    pub jan_qtyz16: [c_char; 16],
    pub _jan_qtyz16: c_char, //잔량
    pub expr_datez10: [c_char; 10],
    pub _expr_datez10: c_char, //만기일
    pub ass_amtz16: [c_char; 16],
    pub _ass_amtz16: c_char, //평가금액
    pub issue_mgamt_ratez6: [c_char; 6],
    pub _issue_mgamt_ratez6: c_char, //종목증거금율         /*float->char*/
    pub medo_slby_amtz16: [c_char; 16],
    pub _medo_slby_amtz16: c_char, //평균매도가
    pub post_lsnpf_amtz16: [c_char; 16],
    pub _post_lsnpf_amtz16: c_char, //매도손익
}
