use std::ffi::{c_char, CStr};

use chrono::FixedOffset;
use encoding::{all::WINDOWS_949, DecoderTrap, Encoding};

lazy_static! {
    pub static ref SEOUL_TZ: FixedOffset = FixedOffset::east_opt(9 * 3600).unwrap();
}

/**
 * 문자열 끝에 null이 없을 수도, 있을 수도 있음
 */
pub fn from_cp949(src: &[c_char]) -> String {
    let mut cloned: Vec<u8> = vec![];
    for s in src.iter() {
        // null을 만나면 여기까지만
        if *s == 0 {
            break;
        }
        cloned.push(*s as u8);
    }
    WINDOWS_949
        .decode(cloned.as_slice(), DecoderTrap::Replace)
        .unwrap()
}

/**
 * 문자열 끝에 null이 있음
 */
pub fn from_cp949_ptr(src: *const c_char) -> String {
    unsafe {
        let cstr: &CStr = CStr::from_ptr(src);
        WINDOWS_949
            .decode(cstr.to_bytes(), DecoderTrap::Replace)
            .unwrap()
    }
}
