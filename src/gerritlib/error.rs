
//! Implements generic error type

use curl;
use git2;
use rustc_serialize;
use std::error::Error;
use std;
use url;

/// Error type
#[derive(Debug)]
pub enum GGRError {
    Curl(curl::Error),
    General(String),
    Git2(git2::Error),
    JsonDecoder(rustc_serialize::json::DecoderError),
    StdIo(std::io::Error),
    Url(url::ParseError),
}

/// General result type
pub type GGRResult<T> = Result<T, GGRError>;

impl From<String> for GGRError {
    fn from(err: String) -> GGRError {
        GGRError::General(err)
    }
}

impl From<git2::Error> for GGRError {
    fn from(err: git2::Error) -> GGRError {
        GGRError::Git2(err)
    }
}

impl From<std::io::Error> for GGRError {
    fn from(err: std::io::Error) -> GGRError {
        GGRError::StdIo(err)
    }
}

impl From<rustc_serialize::json::DecoderError> for GGRError {
    fn from(err: rustc_serialize::json::DecoderError) -> GGRError {
        GGRError::JsonDecoder(err)
    }
}

impl From<curl::Error> for GGRError {
    fn from(err:curl::Error) -> GGRError {
        GGRError::Curl(err)
    }
}

impl From<url::ParseError> for GGRError {
    fn from(err: url::ParseError) -> GGRError {
        GGRError::Url(err)
    }
}

impl GGRError {
    /// error message dispatcher method
    pub fn message(&self) -> &str {
        match *self {
            GGRError::Curl(ref x) => { x.description() },
            GGRError::General(ref x) => { x },
            GGRError::Git2(ref x) => { x.message() },
            GGRError::JsonDecoder(ref x) => { x.description() },
            GGRError::StdIo(ref x) => { x.description() },
            GGRError::Url(ref x) => { x.description() },
        }
    }
}


