
//! Implements generic error type

use curl;
use git2;
use serde_json;
use std;
use url;

#[derive(Debug)]
pub enum GerritError {
    ChangeInputProblem,
    GetReviewerListProblem(String),
    GerritApi(u32, String),
}

quick_error! {
    #[derive(Debug)]
    pub enum GGRError {
        Curl(err: curl::Error) {
            description(err.description())
            from()
        }
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
        JsonError(err: serde_json::error::Error) {
            description(err.description())
            from()
        }
        Num(err: std::num::ParseIntError) {
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
        GerritApiError(err: GerritError) {
        }
    }
}

/// General result type
pub type GGRResult<T> = Result<T, GGRError>;


