
//! Implements generic error type

use git2;
use regex;
use rustc_serialize;
use std;
use url;

quick_error! {
    #[derive(Debug)]
    pub enum GGRError {
        FromUtf8(err: std::string::FromUtf8Error) {
            description(err.description())
            from()
        }
        General(err: String) {
            description(err)
            from()
        }
        Git2(err: git2::Error) {
            description(err.message())
            from()
        }
        JsonDecoder(err: rustc_serialize::json::DecoderError) {
            description(err.description())
            from()
        }
        Num(err: std::num::ParseIntError) {
            description(err.description())
            from()
        }
        Regex(err: regex::Error) {
            description(err.description())
            from()
        }
        StdIo(err: std::io::Error) {
            description(err.description())
            from()
        }
        Url(err: url::ParseError) {
            description(err.description())
            from()
        }
    }
}

/// General result type
pub type GGRResult<T> = Result<T, GGRError>;


