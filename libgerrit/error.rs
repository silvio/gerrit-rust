
//! Implements generic error type

use curl;
use git2;
use serde_json;
use std;
use url;

quick_error! {
    #[derive(Debug)]
    pub enum GerritError {
        UnsupportedVersion(function: String, cur: String, ocu: String) {
            description("Not supported in this version")
                display("function '{}' not implemented in gerrit version '{}' (first occurances in '{}'", function, cur, ocu)
        }

        ChangeInputProblem {
            description("Problem with ChangeInput")
            display("Problem with ChangeInput")
        }

        GetReviewerListProblem(r: String) {
            description("Reviewer List problem")
            display("ProblemWithReviewer: {}", r)
        }

        /// The reviewer isn't found
        ReviewerNotFound {
            description("Reviewer not found")
            display("Reviewer not found")
        }

        GerritApi(status: u32, text: String) {
            description("HTTP problem")
            display("HTTP status: {}, text: {}", status, text.trim())
        }

        NoRevisionInfoEntry {
            description("No revision returned")
            display("No revision returned")
        }

        ChangeIDEmpty {
            description("ChangeID is empty")
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
            display("{}", err)
            from()
        }
    }
}

/// General result type
pub type GGRResult<T> = Result<T, GGRError>;


