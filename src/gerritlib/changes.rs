
use call;
use error::GGRError;
use error::GGRResult;
use gron;
use rustc_serialize;
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

            let data5 = match  rustc_serialize::json::Json::from_str(&data2) {
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

/// Abstraction for Gron format
struct GronInfo {
    sel: String,
    val: String,
}

impl From<String> for GronInfo {
    /// splits a string of 'a=b' style to `GronInfo { sel:"a", val:"b" }`
    fn from(s: String) -> Self {
        let mut out: GronInfo = GronInfo {
            sel: String::new(),
            val: String::new(),
        };
        let mut v = s.splitn(2, '=');
        out.sel = String::from(v.next().unwrap_or("")).trim().to_string();
        out.val = String::from(v.next().unwrap_or("")).trim().to_string();

        out
    }
}

impl GronInfo {
    /// returns true if Self.sel ends with one element of ew
    pub fn sel_ends_with_one_of(&self, ew: &Vec<String>) -> bool {
        for e in ew {
            if self.sel.ends_with(e) {
                return true;
            }
        }
        false
    }
}

#[derive(Default)]
pub struct ChangeInfos {
    pub json: Option<rustc_serialize::json::Json>,
}

impl ChangeInfos {
    /// creates a new ChangeInfos instance. ChangeInfos.json is None
    pub fn new() -> ChangeInfos {
        ChangeInfos {
            json: None,
        }
    }

    /// creates new ChangeInfos object with an initial ChangeInfos.json value
    pub fn new_with_data(json: Option<rustc_serialize::json::Json>)
    -> ChangeInfos {
        ChangeInfos {
            json: json,
        }
    }

    /// returns the self.json as a array of GronInfo objects
    fn as_selectinfo_array(&self) -> Vec<GronInfo> {
        if let Some(json) = self.json.clone() {
            let mut vec = Vec::new();

            if let Err(x) = gron::json_to_gron(&mut vec, "", &json) {
                println!("error: {}", x);
                return Vec::new();
            };

            let v: Vec<GronInfo> = vec.split(|x| *x == '\n' as u8)
                .map(|chunk| {
                    GronInfo::from(String::from_utf8_lossy(chunk).to_string())
                    })
                .collect();
            return v;
        }
        return Vec::new();
    }

    /// returns a Vec<String> object. Only elements there names ends on one of `selector`s entries
    /// are returned. Is a `selector` entry a '*' all fields are returned.
    pub fn as_string(&self, selector: Vec<String>) -> Vec<String> {
        let vec = self.as_selectinfo_array();
        let mut out: Vec<String> = Vec::new();
        let mut all_fields = false;
        let mut with_object_start = false;

        // manipulate selector to have a '.' in front of all variables.
        let selector = {
            let mut sel_v: Vec<String> = Vec::new();
            for s in &selector {
                // special operators
                if s.eq("*") {
                    all_fields = true;
                }
                if s.eq("+") {
                    with_object_start = true;
                }

                let mut s = s.clone();
                s.insert(0, '.');
                sel_v.push(s);
            }
            sel_v
        };

        // find longest 'sel'
        let mut sel_max_len = 0;
        for v in &vec {
            if v.sel.len() > sel_max_len  && (v.sel_ends_with_one_of(&selector) || all_fields) {
                sel_max_len = v.sel.len();
            }
        }

        for v in &vec {
            if v.sel_ends_with_one_of(&selector) || all_fields {
                if v.sel.is_empty() && v.val.is_empty() || (v.val == "{}" && !with_object_start) {
                    continue;
                }
                out.push(format!("{sel:width_sel$} {val}", sel=v.sel, val=v.val, width_sel=sel_max_len));
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
                        rustc_serialize::json::Json::Object(ref x) => {
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

        format!("{}", json.unwrap_or(rustc_serialize::json::Json::String("".into())))
    }

    /// return in human readable form
    pub fn human(&self) -> String {
        let json = self.json.clone();

        format!("{}", json.unwrap_or(rustc_serialize::json::Json::String("".into())).pretty())
    }
}

