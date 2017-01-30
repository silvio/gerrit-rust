
use call;
use error::GGRError;
use error::GGRResult;
use gron::ToGron;
use serde_json;
use regex;
use std::collections::HashMap;
use entities;
use gerrit::GerritAccess;
use url;

/// Interface to retrieve Changes information from gerrit server
pub struct Changes {
    call: call::Call,

    /// Query Information (`q`-parameter) like "owner:self"
    pub querylist: Option<Vec<String>>,

    /// Additional fields, like `DOWNLOAD_COMMANDS` or `CURRENT_ACTIONS`
    pub labellist: Option<Vec<String>>,
}

impl GerritAccess for Changes {
    // creates ("/a/changes", "pp=0&q="querystring"&o=label&o=label")
    fn build_url(&self) -> (String, String) {
        let mut querystring = String::from("pp=0&q=");
        if let Some(ref querylist) = self.querylist {
            let mut fragment = String::new();
            for el in querylist.iter() {
                fragment.push_str(el);
                fragment.push_str("+");
            }
            if let Some(x) = fragment.chars().last() {
                if x == '+' {
                    fragment = fragment.trim_right_matches(x).to_string();
                }
            };

            querystring = format!("{}{}", querystring, fragment);
        };

        if let Some(ref labels) = self.labellist {
            for label in labels {
                querystring = format!("{}&o={}", querystring, label);
            }
        }

        querystring = querystring.replace("//", "/");

        (String::from("/a/changes/"), querystring)
    }
}

impl Changes {
    pub fn new(url: &url::Url) -> Changes {
        Changes {
            call: call::Call::new(url),
            querylist: None,
            labellist: None,
        }
    }

    /// Returns a ChangeInfos object on success. This value comes direct from the http call and
    /// is a valid json object
    pub fn query_changes(&mut self) -> GGRResult<ChangeInfos> {
        let (path, query) = self.build_url();

        self.call.set_url_query(Some(&query));

        match self.call.get(&path) {
            Ok(cr) => {
                let body = match cr.get_body() {
                    Some(x) => x,
                    None => {
                        // no body content
                        return Ok(ChangeInfos::new());
                    }
                };
                let mut data2 = try!(String::from_utf8(body));
                data2 = data2.trim().into();

                let data5 = match serde_json::de::from_str(&data2) {
                    Ok(d) => d,
                    Err(e) => {
                        println!("error: {}",e);
                        return Err(GGRError::from(e));
                    }
                };

                let changeinfos = ChangeInfos::new_with_data(Some(data5));

                Ok(changeinfos)
            },
            Err(x) => {
                Err(GGRError::General(format!("call problem with: {} and {} ({})", path, query, x)))
            }
        }
    }
}

#[derive(Default, Debug, Deserialize)]
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

    pub fn as_string_reg(&self, selectors: &[String]) -> String {
        let json = self.json.clone();

        let mut grondata: Vec<u8> = Vec::new();
        let _ = json.unwrap().to_gron(&mut grondata, "");
        let mut out = String::from("");

        for line in String::from_utf8(grondata).unwrap_or(String::new()).lines() {
            let mut keyval = line.splitn(2, '=');
            let key = keyval.next().unwrap_or("").trim();
            let val = keyval.next().unwrap_or("").trim();

            for selector in selectors {
                if let Ok(re) = regex::Regex::new(selector) {
                    if re.is_match(key) {
                        out.push_str(&format!("{} {}\n", key, val));
                    }
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

        serde_json::ser::to_string_pretty(&json.unwrap_or_else(|| serde_json::value::Value::String("".into()))).unwrap_or_else(|_| "problem with pretty printing".into())
    }

    pub fn to_entities(&self) -> GGRResult<Vec<entities::ChangeInfo>> {
        let json = self.raw();

        match serde_json::de::from_str(&json) {
            Ok(decode) => { Ok(decode) },
            Err(r) => {
                println!("decode error: {}", r);
                println!("json:\n{}", json);
                Err(GGRError::from(r))
            },
        }
    }

    pub fn entity_from_commit(&self, commit: &str) -> GGRResult<entities::ChangeInfo> {
        let entities = try!(self.to_entities());

        for element in entities {
            if let Some(ref revisions) = element.revisions {
                for rev in revisions.keys() {
                    if *rev == commit {
                        return Ok(element.clone());
                    }
                }
            }
        }

        Err(GGRError::General("no entity found".into()))
    }

    /// returns a HashMap with project and tip of a topic.changeset
    pub fn project_tip(&self) -> GGRResult<HashMap<String, String>> {
        let entities = try!(self.to_entities());
        // find involved projects
        let mut list_of_projects = Vec::new();
        for element in &entities {
            let project = &element.project;
            if !list_of_projects.contains(project) {
                list_of_projects.push(project.clone());
            }
        }

        // find tip of every project
        let mut project_tip: HashMap<String, String> = HashMap::new();
        for project in list_of_projects {
            // find in entities the last change of every project for this topic
            let mut list_all_parents = Vec::new();
            for element in &entities {
                if let Some(ref cur_revision) = element.current_revision {
                    if let Some(ref revisions) = element.revisions {
                        if let Some(rev) = revisions.get(cur_revision) {
                            if let Some(ref commit) = rev.commit {
                                if let Some(ref parents) = commit.parents {
                                    for p in parents {
                                        list_all_parents.push(&p.commit);
                                    }
                                }
                            }
                        }
                    }
                }
            }

            for element in &entities {
                if element.project == project {
                    if let Some(ref cur_revision) = element.current_revision {
                        if !list_all_parents.contains(&cur_revision) {
                            // a tip commit is never a parent for a topic
                            project_tip.insert(project, cur_revision.clone());
                            break;
                        }
                    }
                }
            }
        }

        Ok(project_tip)
    }
}

