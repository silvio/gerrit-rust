
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

    /// This function is subject of future changes
    pub fn add_query_part<S>(&mut self, q: S) -> &mut Changes
    where S: Into<String> {
        self.querylist.push(q.into());
        self
    }

    /// This function is subject of future changes
    pub fn add_label<S>(&mut self, l: S) -> &mut Changes
    where S: Into<String> {
        self.labellist.push(l.into());
        self
    }

    /// api function 'GET /changes/'
    ///
    /// This function is subject of future changes
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

    /// api function 'POST /changes/{change-id}/reviewers'
    pub fn add_reviewer(&self, changeid: &str, reviewer: &str) -> GGRResult<entities::AddReviewerResult> {
        if changeid.is_empty() || reviewer.is_empty() {
            return Err(GGRError::GerritApiError(GerritError::GetReviewerListProblem("changeid or reviewer is empty".into())));
        }

        use config;
        let config = config::Config::new(&self.call.get_base());
        if let Ok(version) = config.get_version() {
            let (path, _) = self.build_url();
            let path = format!("{}/{}/reviewers", path, changeid);

            let reviewerinput = if version.starts_with("2.9") {
                entities::ReviewerInput::Gerrit0209(entities::ReviewerInput0209 {
                    reviewer: reviewer.into(),
                    confirmed: None,
                })
            } else if version.starts_with("2.13") {
                entities::ReviewerInput::Gerrit0213(entities::ReviewerInput0213 {
                    reviewer: reviewer.into(),
                    state: None,
                    confirmed: None,
                })
            } else {
                return Err(GGRError::General(format!("Only support for gerit version 2.9 and 2.13. Yor server is {}",version)));
            };

            match self.call.post(&path, &reviewerinput) {
                Ok(cr) => {
                    if cr.ok() {
                        return cr.convert::<entities::AddReviewerResult>();
                    } else {
                        return Err(GGRError::GerritApiError(GerritError::GerritApi(cr.status(), String::from_utf8(cr.get_body().unwrap()).unwrap())));
                    }
                },
                Err(x) => {
                    return Err(GGRError::General(format!("Problem '{}' with add reviewer for {}", x, changeid)));
                }
            }
        }

        return Err(GGRError::General(format!("Could not determine gerrit server version")));
    }

    /// api function 'DELETE /changes/{change-id}/reviewers/{account-id}'
    pub fn delete_reviewer(&self, changeid: &str, reviewer: &str) -> GGRResult<()> {
        if changeid.is_empty() || reviewer.is_empty() {
            return Err(GGRError::GerritApiError(GerritError::GetReviewerListProblem("changeid or reviewer is empty".into())));
        }

        let (path, _) = self.build_url();
        let path = format!("{}/{}/reviewers/{}", path, changeid, reviewer);

        match self.call.delete(&path) {
            Ok(cr) => {
                match cr.status() {
                    200 ... 204 => { return Ok(()); },
                    404 => { return Err(GGRError::GerritApiError(GerritError::ReviewerNotFound)); },
                    _ => { return Err(GGRError::GerritApiError(GerritError::GerritApi(cr.status(), String::from_utf8(cr.get_body().unwrap()).unwrap()))); },
                };
            },
            Err(x) => { return Err(GGRError::General(format!("Problem '{}' with deleting reviewer for {}", x, changeid))); },
        }
    }
}
