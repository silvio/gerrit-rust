//! Handle http requests
//!
//! The http request is handled via external `curl`-command. `curl-rust` has no digest
//! authorization implemented currently. <https://github.com/alexcrichton/curl-rust/issues/120>

use error::GGRError;
use error::GGRResult;
use regex;
use std::fmt;
use std;
use url;

enum CallMethod {
    Get,
}

/// Transform `CallMethod` type to String
impl fmt::Display for CallMethod {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            CallMethod::Get => write!(f, "GET"),
        }
    }
}

/// Representation of a Response
#[derive(Debug)]
pub struct CallResponse {
    /// http return status
    pub status: u32,
    /// header of http response
    pub headers: Vec<String>,
    /// the content
    pub body: Option<Vec<String>>,
}

impl CallResponse {
    /// Creates a new CallResponse object
    ///
    /// This function parsed `curl`s output of a http request
    pub fn new(x: std::process::Output) -> GGRResult<CallResponse> {
        let stdout = try!(String::from_utf8(x.stdout));

        let mut status: u32 = 0;
        let mut body: Vec<String> = Vec::new();
        let mut header: Vec<String> = Vec::new();
        // helper variable to distinguish between body part and header
        let mut bodymode = false;

        let re_statuscode = try!(regex::Regex::new(r"^HTTP/1.1 (.*) .*$"));

        for line in stdout.lines() {
            status = match re_statuscode.captures(line) {
                Some(x) => {
                    if x.is_empty() {
                        status
                    } else {
                        try!(u32::from_str_radix(x.at(1).unwrap(), 10))
                    }
                },
                None => status,
            };

            if status >= 400 { continue; }
            if line.is_empty() { continue; }

            if line.starts_with(")]}'") {
                bodymode = true;
                continue;
            }

            if bodymode {
                body.push(line.to_owned());
            } else {
                header.push(line.to_owned());
            }
        }

        Ok(CallResponse {
            status: status,
            headers: header,
            body: Some(body),
        })
    }
}

/// Abstraction of a HTTP call
pub struct Call {
    url: url::Url,
    username: Option<String>,
    password: Option<String>,
}

impl Call {
    /// Creates a new http call with the base address of a gerrit server
    pub fn new(url: String) -> Call {
        Call {
            url: url::Url::parse(&url).unwrap(),
            username: None,
            password: None,
        }
    }

    pub fn set_credentials<S>(&mut self, username: S, password: S)
    where S: Into<String> {
        self.username = Some(username.into());
        self.password = Some(password.into());
    }

    /// Creates a `CallRequest` object with a specific HTTP method and the complete url
    fn request(&self, method:CallMethod, url:String) -> GGRResult<CallResponse> {
        let userpw = format!("{}:{}", self.username.clone().unwrap_or("".into()), &self.password.clone().unwrap_or("".into()));
        let mut curl_command = std::process::Command::new("curl");
        curl_command.arg("-H").arg("Accept: application/json")
            .arg("--include")
            .arg("--digest")
            .arg("--user").arg(userpw)
            .arg("--request").arg(method.to_string())
            .arg(url);

        let curl_out = try!(curl_command.output());

        if !curl_out.status.success() {
            let errorstrerr = try!(String::from_utf8(curl_out.stderr));
            let errorstrout = try!(String::from_utf8(curl_out.stdout));
            let errormessage = format!("problem with curl command: \nstdout={} \nstderr={}", errorstrout, errorstrerr);
            return Err(GGRError::General(errormessage));
        }

        CallResponse::new(curl_out)
    }

    /// Does a `GET` Request and returns a CallResponse. Only path and querystring is needed. The
    /// base url was setup via [`Call::new()`] function.
    ///
    /// [`Call::new()`]: ./struct.Call.html#method.new
    pub fn get(&self, path: &str, querystring: &str) -> GGRResult<CallResponse> {
        let mut sendurl = self.url.clone();

        let mut path = if self.username.is_some() || self.password.is_some() {
            format!("/a/{}", path)
        } else {
            path.to_string()
        };

        path = path.replace("//", "/");

        sendurl.set_path(&path);
        sendurl.set_query(Some(querystring));

        self.request(CallMethod::Get, sendurl.into_string())
    }
}
