#![allow(non_camel_case_types)]
#![allow(non_upper_case_globals)]
#![allow(non_snake_case)]
#![allow(dead_code)]

use anyhow::{bail, Result};
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int, c_uint};

#[repr(C)]
struct AspellConfig {
    _unused: [u8; 0],
}

#[repr(C)]
struct AspellSpeller {
    _unused: [u8; 0],
}

#[repr(C)]
struct AspellCanHaveError {
    _unused: [u8; 0],
}

#[repr(C)]
struct AspellWordList {
    _unused: [u8; 0],
}

#[repr(C)]
struct AspellStringEnumeration {
    _unused: [u8; 0],
}

extern "C" {
    fn new_aspell_config() -> *mut AspellConfig;
    fn new_aspell_speller(config: *mut AspellConfig) -> *mut AspellCanHaveError;

    fn aspell_config_replace(
        this: *mut AspellConfig,
        key: *const c_char,
        value: *const c_char,
    ) -> c_int;

    fn aspell_error_number(c: *const AspellCanHaveError) -> c_uint;
    fn aspell_error_message(c: *const AspellCanHaveError) -> *const c_char;

    fn to_aspell_speller(c: *mut AspellCanHaveError) -> *mut AspellSpeller;
    fn delete_aspell_config(c: *mut AspellConfig);

    fn delete_aspell_can_have_error(this: *mut AspellCanHaveError);

    fn aspell_speller_check(
        this: *mut AspellSpeller,
        word: *const c_char,
        word_size: c_int,
    ) -> c_int;
    fn aspell_speller_suggest(
        this: *mut AspellSpeller,
        word: *const c_char,
        word_size: c_int,
    ) -> *const AspellWordList;
    fn delete_aspell_speller(this: *mut AspellSpeller);

    fn aspell_word_list_elements(this: *const AspellWordList) -> *mut AspellStringEnumeration;

    fn aspell_string_enumeration_next(this: *mut AspellStringEnumeration) -> *const c_char;
    fn delete_aspell_string_enumeration(this: *mut AspellStringEnumeration);
}

struct SpellerCanHaveError {
    ptr: *mut AspellCanHaveError,
}

impl SpellerCanHaveError {
    fn new(config: *mut AspellConfig) -> Self {
        unsafe {
            Self {
                ptr: new_aspell_speller(config),
            }
        }
    }

    fn error(&self) -> c_uint {
        unsafe { aspell_error_number(self.ptr) }
    }

    fn error_message(&self) -> String {
        unsafe {
            let c_message = aspell_error_message(self.ptr);
            let slice = CStr::from_ptr(c_message);
            let message = slice.to_string_lossy();
            message.to_string()
        }
    }

    fn to_speller(&self) -> Speller {
        unsafe { Speller::new(to_aspell_speller(self.ptr)) }
    }

    // This is not a drop, because the SpellerCanHaveError must
    // live as long as the Speller itself
    fn delete(self) {
        unsafe { delete_aspell_can_have_error(self.ptr) }
    }
}

pub(crate) struct Config {
    ptr: *mut AspellConfig,
}

impl Config {
    pub(crate) fn new() -> Self {
        unsafe {
            Self {
                ptr: new_aspell_config(),
            }
        }
    }

    pub(crate) fn set_lang(&mut self, lang: &str) {
        unsafe {
            let name = CString::new("lang").unwrap();
            let value = CString::new(lang).unwrap();
            aspell_config_replace(self.ptr, name.as_ptr(), value.as_ptr());
        }
    }

    pub(crate) fn speller(&self) -> Result<Speller> {
        let speller_can_have_error = SpellerCanHaveError::new(self.ptr);
        let error = speller_can_have_error.error();
        if error != 0 {
            let message = speller_can_have_error.error_message();
            speller_can_have_error.delete();
            bail!("Could not create speller: {}", message);
        } else {
            Ok(speller_can_have_error.to_speller())
        }
    }
}

impl Drop for Config {
    fn drop(&mut self) {
        unsafe { delete_aspell_config(self.ptr) }
    }
}

#[derive(Debug)]
pub(crate) struct Speller {
    ptr: *mut AspellSpeller,
}

impl Speller {
    fn new(ptr: *mut AspellSpeller) -> Self {
        Self { ptr }
    }

    pub(crate) fn check(&self, word: &str) -> bool {
        let word = CString::new(word).unwrap();
        let n = word.as_bytes().len();
        let c_res = unsafe { aspell_speller_check(self.ptr, word.as_ptr(), n as i32) };
        c_res != 0
    }

    pub(crate) fn suggest(&self, word: &str) -> Vec<String> {
        let word = CString::new(word).unwrap();
        let size = word.as_bytes().len();
        let mut res = vec![];
        unsafe {
            let suggestions = aspell_speller_suggest(self.ptr, word.as_ptr(), size as i32);
            let elements = aspell_word_list_elements(suggestions);
            loop {
                let next = aspell_string_enumeration_next(elements);
                if next.is_null() {
                    break;
                }
                let suggestion = CStr::from_ptr(next);
                res.push(suggestion.to_string_lossy().to_string());
            }
            delete_aspell_string_enumeration(elements);
        }
        res
    }
}

impl Drop for Speller {
    fn drop(&mut self) {
        unsafe {
            delete_aspell_speller(self.ptr);
        }
    }
}

#[cfg(test)]
mod tests;
