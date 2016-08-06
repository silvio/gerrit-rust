
//! Implements the gerrit structure

use call::Call;
use changes;
use error::GGRResult;


/// `Gerrit` structure for management of several gerrit endpoints
pub struct Gerrit {
    call: Call,
}

impl Gerrit {
    /// Creates a new `Gerrit` object
    ///
    /// The url points to the http endpoint of an gerrit server like
    /// `http://localhost:8080/gerrit`. All other function append to this url there endpoint pathes
    /// and query parameters.
    pub fn new<S>(url: S) -> Gerrit
    where S: Into<String> {
        Gerrit {
            call: Call::new(url.into()),
        }
    }

    /// pull changes from gerrit server
    ///
    /// `querylist` is used as filter for the call to gerrit. `additional_infos` gives some more
    /// information of one Change entity.
    pub fn changes(&mut self, querylist: Option<&Vec<String>>, additional_infos: Option<Vec<String>>, username: &str, password: &str)
        -> GGRResult<changes::ChangeInfos>
    {
        let mut querystring = "pp=0&q=".to_string();
        match querylist {
            None => { /* nothing to do, we call without filter */ },
            Some(x) => {
                let urlfragment = Changes::build_url(&x);
                querystring = format!("{}{}", querystring, urlfragment);
            },
        };

        if let Some(labels) = additional_infos {
            if !labels.is_empty() {
                for label in labels {
                    querystring = format!("{}&o={}", querystring, label);
                }
            }
        }

        if !username.is_empty() && !password.is_empty() {
            self.call.set_credentials(username, password);
        }

        changes::Changes::query_changes(&self.call, &querystring)
    }
}

// helper structures
struct Changes;
impl Changes {
    pub fn build_url(querylist: &Vec<String>) -> String {
        let mut out = String::new();
        for el in querylist.iter() {
            out.push_str(el);
            out.push_str("+");
        }
        if let Some(x) = out.chars().last() {
            if x == '+' {
                out = out.trim_right_matches(x).to_string();
            }
        };

        out
    }
}

#[test]
fn test_changes_build_url() {
    assert_eq!(Changes::build_url(&vec!()), "".to_string());
    assert_eq!(Changes::build_url(&vec!("a:1".to_string(), "b:2".to_string())), "a:1+b:2".to_string());
    assert_eq!(Changes::build_url(&vec!("a:1".to_string())), "a:1".to_string());
}
