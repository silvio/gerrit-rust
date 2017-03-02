
use call;
use error::GGRError;
use error::GGRResult;
use error::GerritError;
use entities;
use gerrit::GerritAccess;
use url;

/// Interface to retrieve Changes information from gerrit server
pub struct Changes {
    call: call::Call,

    /// Query Information (`q`-parameter) like "owner:self"
    querylist: Vec<String>,

    /// Additional fields, like `DOWNLOAD_COMMANDS` or `CURRENT_ACTIONS`
    labellist: Vec<String>,
}

impl GerritAccess for Changes {
    // creates ("/a/changes", "pp=0&q="querystring"&o=label&o=label")
    fn build_url(&self) -> (String, String) {
        let mut querystring = String::from("pp=0&q=");
        if ! self.querylist.is_empty() {
            let mut fragment = String::new();
            for el in &self.querylist {
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

        if ! self.labellist.is_empty() {
            for label in &self.labellist {
                querystring = format!("{}&o={}", querystring, &label);
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
            querylist: Vec::new(),
            labellist: Vec::new(),
        }
    }

    pub fn add_query_part<S>(&mut self, q: S) -> &mut Changes
    where S: Into<String> {
        self.querylist.push(q.into());
        self
    }

    pub fn add_label<S>(&mut self, l: S) -> &mut Changes
    where S: Into<String> {
        self.labellist.push(l.into());
        self
    }

    /// api function 'GET /changes/'
    pub fn query_changes(&mut self) -> GGRResult<Vec<entities::ChangeInfo>> {
        let (path, query) = self.build_url();

        self.call.set_url_query(Some(&query));

        match self.call.get(&path) {
            Ok(cr) => {
                if cr.ok() {
                    cr.convert::<Vec<entities::ChangeInfo>>()
                } else {
                    Err(GGRError::GerritApiError(GerritError::GerritApi(cr.status(), String::from_utf8(cr.get_body().unwrap()).unwrap())))
                }
            },
            Err(x) => {
                Err(GGRError::General(format!("call problem with: {} and {} ({})", path, query, x)))
            }
        }
    }

    /// api function 'POST /changes'
    pub fn create_change(&self, ci: &entities::ChangeInput) -> GGRResult<entities::ChangeInfo> {
        if ci.project.is_empty() || ci.branch.is_empty() || ci.subject.is_empty() {
            return Err(GGRError::GerritApiError(GerritError::ChangeInputProblem));
        }

        let (path, _) = self.build_url();

        match self.call.post(&path, &ci) {
            Ok(cr) => {
                if cr.ok() {
                    cr.convert::<entities::ChangeInfo>()
                } else {
                    Err(GGRError::GerritApiError(GerritError::GerritApi(cr.status(), String::from_utf8(cr.get_body().unwrap()).unwrap())))
                }
            },
            Err(x) => {
                Err(GGRError::General(format!("Problem '{}' with change create: {:?}", x, ci)))
            }
        }
    }

    /// api function `GET /changes/{change-id}/reviewers/'
    pub fn get_reviewers(&self, changeid: &str) -> GGRResult<Vec<entities::ReviewerInfo>> {
        if changeid.is_empty() {
            return Err(GGRError::GerritApiError(GerritError::GetReviewerListProblem("changeid is empty".into())));
        }

        let (path, _) = self.build_url();

        let path = format!("{}/{}/reviewers/", path, changeid);

        match self.call.get(&path) {
            Ok(cr) => {
                if cr.ok() {
                    cr.convert::<Vec<entities::ReviewerInfo>>()
                } else {
                    Err(GGRError::GerritApiError(GerritError::GerritApi(cr.status(), String::from_utf8(cr.get_body().unwrap()).unwrap())))
                }
            },
            Err(x) => {
                Err(GGRError::General(format!("Problem '{}' with receiving reviewer list for {}", x, changeid)))
            }
        }
    }
}
