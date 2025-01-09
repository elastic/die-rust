//! Custom error module
//!
use derive_more::{Display, From};

pub type Result<T> = std::result::Result<T, Error>;

#[allow(dead_code)]
#[derive(From, Debug, Display)]
pub enum Error {
    ConversionFailure,
    Overflow,
    Ffi {
        error_code: i32,
    },

    #[from]
    Utf8(std::str::Utf8Error),

    #[from]
    Nul(std::ffi::NulError),
}
