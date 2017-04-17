
use call;
use config;
use error::GGRError;
use error::GGRResult;
use error::GerritError;
use entities;
use gerrit::GerritAccess;
use gerrit::GerritVersion;
use semver;
use serde;
use std;
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
        let mut querystring = String::from("pp=0");
        if ! self.querylist.is_empty() {
            querystring.push_str("&q=");
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

impl GerritVersion for Changes {
    fn check_version(&self, since: String) -> GGRResult<()> {
        let config = config::Config::new(self.call.get_base());
        if let Ok(version) = config.get_version() {
            if semver::Version::parse(&version) < semver::Version::parse(&since) {
                return Err(GGRError::GerritApiError(GerritError::UnsupportedVersion("POST /changes".into(), version.into(), since.into())));
            }
        } else {
            warn!("server version seems not supported, continuing");
        }

        Ok(())
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

    /// generic helper function for calling of call object
    ///
    /// The `desc` parameter is a short description for  error messages, its embedded into 'Problem
    /// '...' with <DESC>'.
    /// The call is executed with the `path` parameter and the `httpmethod` with `uploaddata` for
    /// `Put` and `Post` http methods.
    fn execute<INPUT,OUTPUT>(c: &Changes, desc: &str, path: &str, httpmethod: call::CallMethod, uploaddata: Option<&INPUT>) -> GGRResult<OUTPUT>
    where INPUT: serde::Serialize + std::fmt::Debug,
          OUTPUT: serde::Deserialize
    {
        match c.call.request(httpmethod, path, uploaddata) {
            Ok(cr) => {
                match cr.status() {
                    200 => cr.convert::<OUTPUT>(),
                    status => { Err(GGRError::GerritApiError(GerritError::GerritApi(status, String::from_utf8(cr.get_body().unwrap_or_else(|| "no cause from server".into()))?))) },
                }
            },
            Err(x) => {
                Err(GGRError::General(format!("Problem '{}' with {}", x, desc)))
            }
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

        Changes::execute::<(),Vec<entities::ChangeInfo>>(self, "query change", &path, call::CallMethod::Get, None)
    }

    /// api function 'POST /changes'
    ///
    /// V02.10
    pub fn create_change(&self, ci: &entities::ChangeInput) -> GGRResult<entities::ChangeInfo> {
        if ci.project.is_empty() || ci.branch.is_empty() || ci.subject.is_empty() {
            return Err(GGRError::GerritApiError(GerritError::ChangeInputProblem));
        }

        if let Err(x) = self.check_version("2.10.0".into()) {
            return Err(x);
        }

        let (path, _) = self.build_url();

        Changes::execute(self, "change create", &path, call::CallMethod::Post, Some(&ci))
    }

    /// api function 'GET /changes/{change-id}'
    pub fn get_change(&mut self, changeid: &str, features: Option<Vec<&str>>) -> GGRResult<entities::ChangeInfo> {
        if changeid.is_empty() {
            return Err(GGRError::GerritApiError(GerritError::ChangeIDEmpty));
        }

        if let Some(features) = features {
            for feature in features {
                self.add_label(feature);
            }
        };

        let (path, query) = self.build_url();
        let path = format!("{}/{}", path, changeid);

        self.call.set_url_query(Some(&query));

        Changes::execute::<(),entities::ChangeInfo>(self, "get change", &path, call::CallMethod::Get, None)
    }

    /// api function 'GET /changes/{change-id}/detail'
    pub fn get_change_detail(&self, changeid: &str) -> GGRResult<entities::ChangeInfo> {
        if changeid.is_empty() {
            return Err(GGRError::GerritApiError(GerritError::ChangeIDEmpty));
        }

        let (path, _) = self.build_url();
        let path = format!("{}/{}/detail", path, changeid);

        Changes::execute::<(),entities::ChangeInfo>(self, "get change detail", &path, call::CallMethod::Get, None)
    }

    /// api function `GET /changes/{change-id}/reviewers/'
    pub fn get_reviewers(&self, changeid: &str) -> GGRResult<Vec<entities::ReviewerInfo>> {
        if changeid.is_empty() {
            return Err(GGRError::GerritApiError(GerritError::ChangeIDEmpty));
        }

        let (path, _) = self.build_url();

        let path = format!("{}/{}/reviewers/", path, changeid);

        Changes::execute::<(),Vec<entities::ReviewerInfo>>(self, "receiving reviewer list", &path, call::CallMethod::Get, None)
    }

    /// api function 'POST /changes/{change-id}/reviewers'
    pub fn add_reviewer(&self, changeid: &str, reviewer: &str) -> GGRResult<entities::AddReviewerResult> {
        if changeid.is_empty() || reviewer.is_empty() {
            return Err(GGRError::GerritApiError(GerritError::GetReviewerListProblem("changeid or reviewer is empty".into())));
        }

        let (path, _) = self.build_url();
        let path = format!("{}/{}/reviewers", path, changeid);

        let reviewerinput = entities::ReviewerInput {
                reviewer: reviewer.into(),
                confirmed: None,
                state: None,
        };

        Changes::execute::<&entities::ReviewerInput,entities::AddReviewerResult>(self, "add reviewer", &path, call::CallMethod::Get, Some(&&reviewerinput))
    }

    /// api function 'DELETE /changes/{change-id}/reviewers/{account-id}'
    pub fn delete_reviewer(&self, changeid: &str, reviewer: &str) -> GGRResult<()> {
        if changeid.is_empty() || reviewer.is_empty() {
            return Err(GGRError::GerritApiError(GerritError::GetReviewerListProblem("changeid or reviewer is empty".into())));
        }

        let (path, _) = self.build_url();
        let path = format!("{}/{}/reviewers/{}", path, changeid, reviewer);

        Changes::execute::<(),()>(self, "deleting reviewer", &path, call::CallMethod::Delete, None)
    }

    /// api function 'POST /changes/{change-id}/abandon'
    ///
    /// notify is one of `none`, `owner`, `owner_reviewers` or `all`.
    pub fn abandon_change(&self, changeid: &str, message: Option<&str>, notify: Option<&str>) -> GGRResult<entities::ChangeInfo> {
        if changeid.is_empty() {
            return Err(GGRError::GerritApiError(GerritError::ChangeIDEmpty));
        }

        let (path, _) = self.build_url();
        let path = format!("{}/{}/abandon", path, changeid);

        let notify = match notify {
            Some(notify) => {
                match notify {
                    "all" => Some(entities::AbandonInputNotify::ALL),
                    "owner" => Some(entities::AbandonInputNotify::OWNER),
                    "owner_reviewer" => Some(entities::AbandonInputNotify::OWNER_REVIEWERS),
                    _ => Some(entities::AbandonInputNotify::NONE),
                }
            },
            None => None
        };

        let abandoninput = entities::AbandonInput {
                message: message.map(|s| s.to_string()),
                notify: notify,
        };

        Changes::execute::<&entities::AbandonInput,entities::ChangeInfo>(self, "abandon change", &path, call::CallMethod::Post, Some(&&abandoninput))
    }

    /// api function 'POST /changes/{change-id}/restore'
    pub fn restore_change(&self, changeid: &str, message: Option<&str>) -> GGRResult<entities::ChangeInfo> {
        if changeid.is_empty() {
            return Err(GGRError::GerritApiError(GerritError::ChangeIDEmpty));
        }

        let (path, _) = self.build_url();
        let path = format!("{}/{}/restore", path, changeid);

        let restoreinput = entities::RestoreInput {
            message: message.map(|s| s.to_string()),
        };

        Changes::execute::<&entities::RestoreInput,entities::ChangeInfo>(self, "restore change", &path, call::CallMethod::Post, Some(&&restoreinput))
    }

    /// api function 'POST /changes/{change-id}/revisions/{revision-id}/review'
    pub fn set_review(&self, changeid: &str, revisionid: &str, message: Option<&str>, labels: Option<entities::ReviewInfo>) -> GGRResult<entities::ReviewInfo> {
        if changeid.is_empty() || revisionid.is_empty() {
            return Err(GGRError::GerritApiError(GerritError::ChangeIDEmpty));
        }

        let (path, _) = self.build_url();
        let path = format!("{}/{}/revisions/{}/review", path, changeid, revisionid);

        use std::collections::HashMap;

        #[derive(Serialize, Debug)]
        struct Review {
            message: Option<String>,
            labels: HashMap<String, i8>,
        };

        let review = Review {
            message: message.map(|s| s.to_string()),
            labels: labels.unwrap_or(entities::ReviewInfo{ labels: HashMap::new() }).labels,
        };

        Changes::execute::<&Review,entities::ReviewInfo>(self, "set review", &path, call::CallMethod::Post, Some(&&review))
    }
}
