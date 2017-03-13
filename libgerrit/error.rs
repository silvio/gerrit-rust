
//! Implements generic error type

use curl;
use git2;
use serde_json;
use std;
use url;

quick_error! {
    #[derive(Debug)]
    pub enum GerritError {
        ChangeInputProblem {
            display("Problem with ChangeInput")
        }

        GetReviewerListProblem(r: String) {
            display("ProblemWithReviewer: {}", r)
        }

        /// The reviewer isn't found
        ReviewerNotFound {
            description("Reviewer not found")
            display("Reviewer not found")
        }

        GerritApi(status: u32, text: String) {
            display("HTTP status: {}, text: {}", status, text)
        }

        NoRevisionInfoEntry {
            display("No revision returned")
        }
    }
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
        HTTPError(status: u32) {
            description("HTTP Error")
            display("HTTP not success ({})", status)
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
            from()
        }
    }
}

/// General result type
pub type GGRResult<T> = Result<T, GGRError>;


