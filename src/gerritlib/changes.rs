
use call;
use error::GGRError;
use error::GGRResult;
use rustc_serialize;
use std::collections::HashMap;


pub struct Changes { }

impl Changes {
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

#[derive(Default)]
pub struct ChangeInfos {
    pub json: Option<rustc_serialize::json::Json>,
}

impl ChangeInfos {
    pub fn new() -> ChangeInfos {
        ChangeInfos {
            json: None,
        }
    }

    pub fn new_with_data(json: Option<rustc_serialize::json::Json>)
    -> ChangeInfos {
        ChangeInfos {
            json: json,
        }
    }

    /// returns a String object of all objects and the needed fields.
    pub fn as_string(&self, fields: &[String]) -> String {
        let mut out = String::new();
        if let Some(obj) = self.json.clone() {
            if let Some(array) = obj.as_array() {
                for entry in array {
                    let mut line = String::new();
                    for field in fields {
                        if let Some(element) = entry.find(field) {
                            let el = self.json_to_string(element);

                            line.push_str(&el);
                            line.push_str(" | ");
                        }
                    }
                    out.push_str(line.trim_right_matches(" | "));
                    out.push('\n');
                }
                out = out.trim_right_matches('\n').to_string();
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

    fn json_to_string(&self, j: &rustc_serialize::json::Json) -> String {
        let el = match *j {
            rustc_serialize::json::Json::I64(x) => { format!("{}", x) },
            rustc_serialize::json::Json::U64(x) => { format!("{}", x) },
            rustc_serialize::json::Json::F64(x) => { format!("{}", x) },
            rustc_serialize::json::Json::String(ref x) => { format!("{}", x) },
            rustc_serialize::json::Json::Boolean(x) => { format!("{}", x) },
            rustc_serialize::json::Json::Array(ref x) => {
                let mut out = String::from("[");
                for xel in x {
                    format!("{},{}", out, self.json_to_string(xel));
                }
                out.push_str("]");
                out
            },
            rustc_serialize::json::Json::Object(ref x) => {
                // TODO: json parsing of a object, x is a BtreeMap type
                format!("{:?}", x)
            },
            rustc_serialize::json::Json::Null => { "N/A".into() },
        };

        String::from(el)
    }
}

