//! Handle http requests
//!
//! The http request is handled via external `curl`-command. `curl-rust` has no digest
//! authorization implemented currently. <https://github.com/alexcrichton/curl-rust/issues/120>

use curl;
use error::GGRResult;
use error::GGRError;
use std::cell::RefCell;
use std::cell::RefMut;
use std::fmt;
use std::io::Read;
use std::io::Write;
use url;

/// helper function for Request without body content and with content
fn send_req<W: Write>(handle: &mut curl::easy::Easy,
                     out: &mut W,
                     body: Option<Vec<u8>>)
    -> GGRResult<(u32, Vec<String>)> {
        match body {
            Some(body) => {
                let mut body = &body[..];
                try!(handle.upload(true));
                try!(handle.in_filesize(body.len() as u64));
                handle_req(handle, out, &mut |buf| body.read(buf).unwrap_or(0))
            },
            None => {
                handle_req(handle, out, &mut |_| 0)
            }
        }
}

/// Does the real send
fn handle_req<W: Write>(handle: &mut curl::easy::Easy,
                       out: &mut W,
                       read: &mut FnMut(&mut [u8]) -> usize)
    -> GGRResult<(u32, Vec<String>)> {
        let mut headers = Vec::new();

        {
            let mut handle = handle.transfer();

            try!(handle.read_function(|buf| Ok(read(buf))));
            try!{handle.write_function(|data| {
                Ok(match out.write_all(data) {
                    Ok(_) => data.len(),
                    Err(_) => 0,
                })
            })};

            try!{handle.header_function(|data| {
                headers.push(String::from_utf8_lossy(data).into_owned());
                true
            })};

            try!(handle.perform());
        }

        Ok((try!{handle.response_code()}, headers))
}

pub enum CallMethod {
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

/// Represents a HTTP request
///
/// For usage we fill the `CallRequest` with an header and a body. Later we call via `handler` the
/// gerrit server.
pub struct CallRequest<'a> {
    handle: RefMut<'a, curl::easy::Easy>,
    headers: curl::easy::List,
    body: Option<Vec<u8>>,
}

impl<'a> CallRequest<'a> {
    /// creates new instance of [`CallRequest`] with a curl-handler, the HTTP method and the url
    /// endpoint
    pub fn new(mut handle: RefMut<'a, curl::easy::Easy>,
               method: CallMethod,
               url: String)
        -> GGRResult<CallRequest<'a>> {
            let mut headers = curl::easy::List::new();
            // we only want compact json data
            let _ = headers.append("Accept: application/json");

            match method {
                CallMethod::Get => try!{handle.get(true)},
            };

            try!(handle.url(&url));

            Ok(CallRequest {
                handle: handle,
                headers: headers,
                body: None,
            })
    }

    /// helper function to handle a request, write the returned content to a `Write` capabel object
    /// and returns a `CallResponse`.
    fn send_into<W:Write>(mut self, out: &mut W) -> GGRResult<CallResponse> {
        try!(self.handle.http_headers(self.headers));

        let (status, headers) = try!(send_req(&mut self.handle, out, self.body));

        Ok(CallResponse {
            status: status,
            headers: headers,
            body: None,
        })
    }

    /// Does the send function.
    ///
    /// **NOTICE**: The first 4 characters are cutted from the returned content. We want only json
    /// data which has a prevention against XSSI attacks. More here:
    /// <https://gerrit-documentation.storage.googleapis.com/Documentation/2.12.3/rest-api.html#output>
    pub fn send(self) -> GGRResult<CallResponse> {
        let mut out: Vec<u8> = Vec::new();
        let mut rv = try!(self.send_into(&mut out));

        // TODO: log this with a logging facility
        // println!("return-from-server: {:?}", rv);

        /* cut first 4 bytes from output stream */
        if out.starts_with(b")]}'") {
            out = out[4..].into();
        }

        rv.body = Some(out);
        Ok(rv)
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
    pub body: Option<Vec<u8>>,
}

/// Abstraction of a HTTP call
pub struct Call {
    url: url::Url,
    handle: RefCell<curl::easy::Easy>,
    username: Option<String>,
    password: Option<String>,
}

impl Call {
    /// Creates a new http call with the base address of a gerrit server
    pub fn new(url: String) -> Call {
        Call {
            url: url::Url::parse(&url).unwrap(),
            handle: RefCell::new(curl::easy::Easy::new()),
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
    fn request(&self, method:CallMethod, url:String) -> GGRResult<CallRequest> {
        let mut handle = self.handle.borrow_mut();

        if self.username.is_some() && self.password.is_some() {
            try!(handle.cookie_session(true));
            try!(handle.username(&self.username.clone().unwrap()));
            try!(handle.password(&self.password.clone().unwrap()));
        }


        CallRequest::new(handle, method, url)
    }

    /// Does a `GET` Request and returns a CallResponse. Only path and querystring is needed. The
    /// base url was setup via [`Call::new()`] function.
    ///
    /// [`Call::new()`]: ./struct.Call.html#method.new
    pub fn get(&self, path: &str, querystring: &str) -> GGRResult<CallResponse> {
        let mut sendurl = self.url.clone();

        let mut path = if self.username.is_some() || self.password.is_some() {
            format!("{}/a/{}", sendurl.path().to_string(), path)
        } else {
            path.to_string()
        };

        path = path.replace("//", "/");

        sendurl.set_path(&path);
        sendurl.set_query(Some(querystring));

        // TODO: log this with a logging facility
        //println!("url: {:?}", sendurl);

        for am in vec!(
            curl::easy::Auth::new().digest(true),
            curl::easy::Auth::new().basic(true)
        ) {
            let mut call_request = try!(self.request(CallMethod::Get, sendurl.to_owned().into_string()));
            try!(call_request.handle.http_auth(am));

            let call_response = try!(call_request.send());
            if call_response.status == 401 {
                continue;
            }

            return Ok(call_response);
        }

        Err(GGRError::General("No Authentication algorithm found for your gerrit server. 'basic' and 'digest' tested".into()))
    }
}
