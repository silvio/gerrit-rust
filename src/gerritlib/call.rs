use std::io::{Read, Write};
use std::fmt;
use std::cell::{RefMut, RefCell};
use std::ascii::AsciiExt;

use serde::{Serialize, Deserialize};
use serde_json;
use curl;
use url;

use error::GGRResult;
use error::GGRError;

fn send_req<W: Write>(handle: &mut curl::easy::Easy,
                      out: &mut W,
                      body: Option<Vec<u8>>)
                      -> GGRResult<(u32, Vec<String>)> {
    match body {
        Some(body) => {
            let mut body = &body[..];
            handle.upload(true)?;
            handle.in_filesize(body.len() as u64)?;
            handle_req(handle, out, &mut |buf| body.read(buf).unwrap_or(0))
        }
        None => handle_req(handle, out, &mut |_| 0),
    }
}

fn handle_req<W: Write>(handle: &mut curl::easy::Easy,
                        out: &mut W,
                        read: &mut FnMut(&mut [u8]) -> usize)
                        -> GGRResult<(u32, Vec<String>)> {
    let mut headers = Vec::new();
    {
        let mut handle = handle.transfer();
        handle.read_function(|buf| Ok(read(buf)))?;
        handle.write_function(|data| {
                Ok(match out.write_all(data) {
                    Ok(_) => data.len(),
                    Err(_) => 0,
                })
            })?;
        handle.header_function(|data| {
                headers.push(String::from_utf8_lossy(data).into_owned());
                true
            })?;
        handle.perform()?;
    }

    Ok((handle.response_code()?, headers))
}

#[derive(PartialEq, Debug)]
enum CallMethod {
    Get,
    Post,
    Put,
    Delete,
}

impl fmt::Display for CallMethod {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            CallMethod::Get => write!(f, "GET"),
            CallMethod::Post => write!(f, "POST"),
            CallMethod::Put => write!(f, "PUT"),
            CallMethod::Delete => write!(f, "DELETE"),
        }
    }
}

pub struct Call {
    shared_handle: RefCell<curl::easy::Easy>,
    base: url::Url,
}

impl Call {
    pub fn new(url: &url::Url) -> Call {
        Call {
            shared_handle: RefCell::new(curl::easy::Easy::new()),
            base: url.clone(),
        }
    }

    pub fn set_url_query(&mut self, q: Option<&str>) {
        self.base.set_query(q);
    }

    // Low Level Methods

    fn do_request(&self, method: &CallMethod, url: &str) -> GGRResult<CallRequest> {
        let mut handle = self.shared_handle.borrow_mut();
        try!(handle.cookie_session(true));
        try!(handle.netrc(curl::easy::NetRc::Required));

        CallRequest::new(handle, method, url)
    }

    fn request<S: Serialize>(&self, method: CallMethod, path: &str, body: Option<&S>) -> GGRResult<CallResponse> {
        let mut sendurl = self.base.clone();
        // double replace for pathes with three ///.
        let complete_path = format!("{}/{}", sendurl.path(), path).replace("//", "/").replace("//", "/");
        sendurl.set_path(&complete_path);

        debug!("url-to-send: {:?}", sendurl);

        for am in vec!(
            curl::easy::Auth::new().digest(true),
            curl::easy::Auth::new().basic(true),
        ) {
            let mut call_request = try!(self.do_request(&method, &sendurl.to_owned().into_string()));
            if let Some(body) = body {
                call_request.with_json_body(&body).ok();
            }

            try!(call_request.handle.http_auth(am));
            let call_response = try!(call_request.send());
            if call_response.status == 401 {
                continue;
            }
            return Ok(call_response);
        }
        Err(GGRError::General("No Authentication algorithm found for your gerrit server. 'basic' and 'digest' tested".into()))
    }

    /// Convenience method that performs a `GET` request.
    pub fn get(&self, path: &str) -> GGRResult<CallResponse> {
        self.request::<String>(CallMethod::Get, path, None)
    }

    /// Convenience method that performs a `DELETE` request.
    pub fn delete(&self, path: &str) -> GGRResult<CallResponse> {
        self.request::<String>(CallMethod::Delete, path, None)
    }

    /// Convenience method that performs a `POST` request with JSON data.
    pub fn post<S: Serialize>(&self, path: &str, body: &S) -> GGRResult<CallResponse> {
        self.request(CallMethod::Post, path, Some(body))
    }

    /// Convenience method that performs a `PUT` request with JSON data.
    pub fn put<S: Serialize>(&self, path: &str, body: &S) -> GGRResult<CallResponse> {
        self.request(CallMethod::Put, path, Some(body))
    }
}

/// Iterator over response headers
#[allow(dead_code)]
pub struct Headers<'a> {
    lines: &'a [String],
    idx: usize,
}

impl<'a> Iterator for Headers<'a> {
    type Item = (&'a str, &'a str);

    fn next(&mut self) -> Option<(&'a str, &'a str)> {
        self.lines.get(self.idx).map(|line| {
            self.idx += 1;
            match line.find(':') {
                Some(i) => (&line[..i], line[i + 1..].trim()),
                None => (line[..].trim(), ""),
            }
        })
    }
}

pub struct CallRequest<'a> {
    handle: RefMut<'a, curl::easy::Easy>,
    headers: curl::easy::List,
    body: Option<Vec<u8>>,
}

impl<'a> CallRequest<'a> {
    fn new(mut handle: RefMut<'a, curl::easy::Easy>,
           method: &CallMethod,
           url: &str)
           -> GGRResult<CallRequest<'a>> {
        info!("request {} {}", method, url);

        let mut headers = curl::easy::List::new();
        headers.append("Accept: application/json").ok();

        match *method {
            CallMethod::Get => try!(handle.get(true)),
            CallMethod::Post => try!(handle.custom_request("POST")),
            CallMethod::Put => try!(handle.custom_request("PUT")),
            CallMethod::Delete => try!(handle.custom_request("DELETE")),
        }

        handle.url(url)?;

        Ok(CallRequest {
            handle: handle,
            headers: headers,
            body: None,
        })
    }

    /// adds a specific header to the request
    pub fn with_header(mut self, key: &str, value: &str) -> GGRResult<CallRequest<'a>> {
        self.headers.append(&format!("{}: {}", key, value))?;
        Ok(self)
    }

    /// sets the JSON request body for the request.
    pub fn with_json_body<S: Serialize>(&mut self, body: &S) -> GGRResult<&mut CallRequest<'a>> {
        let mut body_bytes: Vec<u8> = vec![];
        serde_json::to_writer(&mut body_bytes, &body)?;
        info!("sending JSON data ({} bytes)", body_bytes.len());
        self.body = Some(body_bytes);
        self.headers.append("Content-Type: application/json")?;
        Ok(self)
    }

    /// attaches some form data to the request.
    pub fn with_form_data(&mut self, form: curl::easy::Form) -> GGRResult<&mut CallRequest<'a>> {
        info!("sending form data");
        self.handle.httppost(form)?;
        self.body = None;
        Ok(self)
    }

    /// enables or disables redirects.  The default is off.
    pub fn follow_location(&mut self, val: bool) -> GGRResult<&mut CallRequest<'a>> {
        info!("follow redirects: {}", val);
        self.handle.follow_location(val)?;
        Ok(self)
    }

    /// Sends the request and writes response data into the given file
    /// instead of the response object's in memory buffer.
    pub fn send_into<W: Write>(mut self, out: &mut W) -> GGRResult<CallResponse> {
        self.handle.http_headers(self.headers)?;
        let local_body = self.body.clone();
        let (status, headers) = send_req(&mut self.handle, out, local_body)?;
        info!("response: {}", status);
        Ok(CallResponse {
            status: status,
            headers: headers,
            body: None,
        })
    }

    /// Sends the request and reads the response body into the response object.
    pub fn send(self) -> GGRResult<CallResponse> {
        let mut out = vec![];
        let mut rv = self.send_into(&mut out)?;

        debug!("return-from-server: {:?}", rv);

        // cut first 4 bytes from output stream
        // *NOTICE**: The first 4 characters are cutted from the returned content. We want only
        // json data which has a prevention against XSSI attacks. More here:
        // <https://gerrit-documentation.storage.googleapis.com/Documentation/2.12.3/rest-api.html#output>
        if out.starts_with(b")]}'") {
            out = out[4..].into();
        }

        rv.body = Some(out);
        Ok(rv)
    }
}

#[derive(Clone, Debug)]
pub struct CallResponse {
    status: u32,
    headers: Vec<String>,
    body: Option<Vec<u8>>,
}

impl CallResponse {
    /// Returns the status code of the response
    pub fn status(&self) -> u32 {
        self.status
    }

    /// Indicates that the request failed
    pub fn failed(&self) -> bool {
        self.status >= 400 && self.status <= 600
    }

    /// Indicates that the request succeeded
    pub fn ok(&self) -> bool {
        !self.failed()
    }

    /// Converts the response into a result object.  This also converts non okay response codes
    /// into errors.
    pub fn to_result(&self) -> GGRResult<&CallResponse> {
        debug!("headers:");
        for (header_key, header_value) in self.headers() {
            if !header_value.is_empty() {
                debug!("  {}: {}", header_key, header_value);
            }
        }
        if let Some(ref body) = self.body {
            debug!("body: {}", String::from_utf8_lossy(body));
        }
        if self.ok() {
            return Ok(self);
        }
        Err(GGRError::General(format!("generic error: {}", self.status())))
    }

    /// Deserializes the response body into the given type
    pub fn deserialize<T: Deserialize>(&self) -> GGRResult<T> {
        Ok(serde_json::from_reader(match self.body {
            Some(ref body) => body,
            None => &b""[..],
        })?)
    }

    /// Like `deserialize` but consumes the response and will convert
    /// failed requests into proper errors.
    pub fn convert<T: Deserialize>(self) -> GGRResult<T> {
        self.to_result().and_then(|x| x.deserialize())
    }

    /// Iterates over the headers.
    #[allow(dead_code)]
    pub fn headers(&self) -> Headers {
        Headers {
            lines: &self.headers[..],
            idx: 0,
        }
    }

    /// Looks up the first matching header for a key.
    #[allow(dead_code)]
    pub fn get_header(&self, key: &str) -> Option<&str> {
        for (header_key, header_value) in self.headers() {
            if header_key.eq_ignore_ascii_case(key) {
                return Some(header_value);
            }
        }
        None
    }

    pub fn get_body(&self) -> Option<Vec<u8>> {
        self.body.clone()
    }
}
