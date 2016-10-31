
use call;
use error::GGRError;
use error::GGRResult;
use gron;
use serde_json;
use regex;
use std::collections::HashMap;


pub struct Changes { }

impl Changes {
    /// Returns a ChangeInfos object on success. This value comes direct from the http call and
    /// is a valid json object
    pub fn query_changes(call: &call::Call, querystring: &str) -> GGRResult<ChangeInfos> {
        if let Ok(cr) = call.get("/changes/".into(), querystring) {
            let body = match cr.body {
                Some(x) => x,
                None => {
                    /* no body content */
                    return Ok(ChangeInfos::new());
                }
            };
            let data2 = try!(String::from_utf8(body));

            let data5 = match serde_json::de::from_str(&data2) {
                Ok(d) => d,
                Err(e) => {
                    println!("error: {}",e);
                    return Err(GGRError::from(e));
                }
            };

            let changeinfos = ChangeInfos::new_with_data(Some(data5));

            return Ok(changeinfos);
        } else {
            println!("call problem with: {}", &querystring);
        }
        Ok(ChangeInfos::new())
    }
}

/// Abstraction for Gron format into a Key/Val storage tyupe
pub struct KeyValue {
    pub id: String,
    pub key: String,
    pub val: String,
}

impl From<String> for KeyValue {
    /// splits a string of 'a=b' style to `KeyValue { sel:"a", val:"b" }`
    fn from(s: String) -> Self {
        let mut out: KeyValue = KeyValue {
            id: String::new(),
            key: String::new(),
            val: String::new(),
        };

        let re = regex::Regex::new(r"^\[(?P<id>\di*?)].(?P<key>.*)=(?P<val>.*)$").unwrap();
        for cap in re.captures_iter(&s) {
            out.id = cap.name("id").unwrap_or("").trim().into();
            out.key = cap.name("key").unwrap_or("").trim().into();
            out.val = cap.name("val").unwrap_or("").trim().into();
        }

        out
    }
}

impl KeyValue {
    /// returns true if Self.key ends with one element of ew
    fn key_end_with_one_of(&self, ew: &[String]) -> bool {
        for e in ew {
            if self.key.ends_with(e) {
                return true;
            }
        }
        false
    }
}

#[derive(Default, Debug)]
pub struct ChangeInfos {
    pub json: Option<serde_json::value::Value>,
    filter_key: Vec<String>,
    filter_val: Vec<String>,
}

impl ChangeInfos {
    /// creates a new ChangeInfos instance. ChangeInfos.json is None
    pub fn new() -> ChangeInfos {
        ChangeInfos {
            json: None,
            filter_key: Vec::new(),
            filter_val: Vec::new(),
        }
    }

    /// creates new ChangeInfos object with an initial ChangeInfos.json value
    pub fn new_with_data(json: Option<serde_json::value::Value>)
    -> ChangeInfos {
        ChangeInfos {
            json: json,
            filter_key: Vec::new(),
            filter_val: Vec::new(),
        }
    }

    /// returns the self.json as a array of KeyValue objects
    ///
    /// The returned array is filtered through `filter_key`, `filter_val` function
    pub fn as_keyval_array(&self) -> Vec<KeyValue> {
        let mut out = Vec::new();
        if let Some(json) = self.json.clone() {
            let mut vec = Vec::new();

            if let Err(x) = gron::json_to_gron(&mut vec, "", &json) {
                println!("error: {}", x);
                return Vec::new();
            };

            for line in vec.split(|x| *x == b'\n') {
                let kv = KeyValue::from(String::from_utf8_lossy(line).to_string());

                let mut matched = false;
                // key filter
                for r in &self.filter_key {
                    if let Ok(re) = regex::Regex::new(r) {
                        if re.is_match(&kv.key) {
                            matched = true;
                            break;
                        }
                    }
                }

                // val filter
                for r in &self.filter_val {
                    if let Ok(re) = regex::Regex::new(r) {
                        if re.is_match(&kv.val) {
                            matched = true;
                            break;
                        }
                    }
                }

                if matched {
                    out.push(kv);
                }
            }
        }
        out
    }

    /// add a regular expression filter for keys
    ///
    /// The filter needs to be resetted through `filter_reset`.
    pub fn filter_key(&mut self, r: &str) -> &mut Self {
        self.filter_key.push(String::from(r));
        self
    }

    /// add a regular expression filter for values
    ///
    /// The filter needs to be resetted through `filter_reset`.
    pub fn filter_val(&mut self, r: String) -> &mut Self {
        self.filter_val.push(r);
        self
    }

    /// reset key and value filter
    pub fn filter_reset(&mut self) -> &mut Self {
        self.filter_val.clear();
        self.filter_key.clear();
        self
    }

    /// returns a Vec<String> object. Only elements there names ends on one of `selector`s entries
    /// are returned. Is a `selector` entry a '*' all fields are returned.
    pub fn as_string(&self, selector: Vec<String>) -> String {
        let vec = self.as_keyval_array();
        let mut out = String::new();
        let mut all_fields = false;
        let mut with_object_start = false;

        let selector = {
            let mut sel_v: Vec<String> = Vec::new();
            for s in &selector {
                // special operators
                if s.eq("*") {
                    all_fields = true;
                    continue;
                }
                if s.eq("+") {
                    with_object_start = true;
                    continue;
                }
                if s.eq("[]") {
                    continue;
                }

                // manipulate selector to have a '.' in front of all variables.
                let mut s = s.clone();
                s.insert(0, '.');
                sel_v.push(s);
            }
            sel_v
        };

        // find longest 'sel'
        let mut sel_max_len = 0;
        for v in &vec {
            if v.key.len() > sel_max_len  && (v.key_end_with_one_of(&selector) || all_fields) {
                sel_max_len = v.key.len();
            }
        }

        for v in &vec {
            if v.key_end_with_one_of(&selector) || all_fields {
                if v.key.is_empty() && v.val.is_empty() || (v.val == "{}" && !with_object_start) {
                    continue;
                }
                out.push_str(&format!("{sel:width_sel$} {val}\n", sel=v.key, val=v.val, width_sel=sel_max_len));
            }
        }

        out
    }

    pub fn as_string_reg(&self, selector: &str) -> Vec<String> {
        let vec = self.as_keyval_array();
        let mut out: Vec<String> = Vec::new();

        if let Ok(re) = regex::Regex::new(selector) {
            for kv in vec {
                if re.is_match(&kv.key) {
                    out.push(format!("{} {}", kv.key, kv.val));
                }
            }
        }

        out
    }

    /// prints all selectable fields os a search string
    ///
    /// returns two values. First one is the count of returned json objects and second value is a
    /// HashMap<String, usize> with all fields and gow much they occure.
    pub fn fieldslist(&self) -> (usize, HashMap<String, usize>) {
        let mut out_hmap: HashMap<String, usize> = HashMap::new();
        let mut entries = 0;

        if let Some(obj) = self.json.clone() {
            if let Some(array) = obj.as_array() {
                entries = array.len();
                for entry in array {
                    match *entry {
                        serde_json::value::Value::Object(ref x) => {
                            for key in x.keys() {
                                let counter = out_hmap.entry(key.to_owned()).or_insert(0);
                                *counter += 1;
                            }
                        }
                        _ => continue,
                    }
                }
            } else {
                println!("no array");
            }
        }

        (entries, out_hmap)
    }

    /// return the string in machinereadable format
    pub fn raw(&self) -> String {
        let json = self.json.clone();

        format!("{}", json.unwrap_or(serde_json::value::Value::String("".into())))
    }

    /// return in human readable form
    pub fn human(&self) -> String {
        let json = self.json.clone();

        format!("{}", &serde_json::ser::to_string_pretty(&json.unwrap_or(serde_json::value::Value::String("".into()))).unwrap_or("problem with pretty printing"))
    }
}

