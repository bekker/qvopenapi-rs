use std::ffi::{c_char, CStr};

use chrono::FixedOffset;
use encoding::{all::WINDOWS_949, DecoderTrap, Encoding};

use crate::error::*;

lazy_static! {
    pub static ref SEOUL_TZ: FixedOffset = FixedOffset::east_opt(9 * 3600).unwrap();
}

/**
 * "100%  " -> 100.0
 */
pub fn parse_ratio_str(src: &[c_char]) -> Result<Option<f64>, QvOpenApiError> {
    let input = parse_string(src)?;

    if input.is_empty() {
        return Ok(None);
    }

    if input.ends_with('%') {
        let substr: &str = &input[..input.len() - 1];
        let parsed: f64 = substr
            .parse()
            .map_err(|_| QvOpenApiError::ParseRatioError { input })?;
        Ok(Some(parsed))
    } else {
        Err(QvOpenApiError::ParseRatioError { input })
    }
}

/**
 * "    1226" -> 12.26
 */
pub fn parse_ratio(src: &[c_char]) -> Result<Option<f64>, QvOpenApiError> {
    Ok(parse_number(src)?.map(|num| num as f64 / 100.))
}

/**
 * "    1234" -> 1234
 * "00001234" -> 1234
 * "-0001234" -> -1234
 */
pub fn parse_number(src: &[c_char]) -> Result<Option<i64>, QvOpenApiError> {
    let input = parse_string(src)?;

    if input.is_empty() {
        return Ok(None);
    }

    let mut filtered = String::new();
    let mut is_minus = false;
    for (_, ch) in input.chars().enumerate() {
        if ch == '-' {
            if filtered.is_empty() && !is_minus {
                is_minus = true;
                continue;
            } else {
                return Err(QvOpenApiError::ParseNumberError { input });
            }
        }

        if (ch == '0' || ch == ' ') && filtered.is_empty() {
            continue;
        }

        if !ch.is_numeric() {
            return Err(QvOpenApiError::ParseNumberError { input });
        }

        filtered.push(ch);
    }

    if filtered.is_empty() {
        return Ok(Some(0));
    }

    let parsed: Result<i64, QvOpenApiError> = filtered
        .parse()
        .map_err(|_| QvOpenApiError::ParseNumberError { input })
        .map(|num: i64| if is_minus { -num } else { num });

    Ok(Some(parsed?))
}

pub fn parse_string(src: &[c_char]) -> Result<String, QvOpenApiError> {
    Ok(from_cp949(&src).trim().to_string())
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
