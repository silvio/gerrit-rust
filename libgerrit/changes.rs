
use call;
use config;
use error::GGRError;
use error::GGRResult;
use error::GerritError;
use entities;
use serde;
use std;
use url;

const ENDPOINT: &'static str = "/a/changes";

/// Interface to retrieve Changes information from gerrit server
pub struct Changes {
    call: call::Call,
}

impl<'de> Changes {
    pub fn new(url: &url::Url) -> Changes {
        Changes {
            call: call::Call::new(url),
        }
    }

    fn build_query_string<S>(querylist: Option<Vec<S>>) -> String
        where S: Into<String>  {
        let mut querystring = String::new();
        if let Some(querylist) = querylist {
            if ! querylist.is_empty() {
                querystring.push_str("&q=");
                let mut fragment = String::new();
                for el in querylist {
                    fragment.push_str(&el.into()[..]);
                    fragment.push_str("+");
                }
                if let Some(x) = fragment.chars().last() {
                    if x == '+' {
                        fragment = fragment.trim_right_matches(x).to_string();
                    }
                };

                querystring = format!("{}{}", querystring, fragment);
            };
        };

        debug!("query-string: '{}'", querystring);
        querystring
    }

    fn build_label_string<S>(labellist: Option<Vec<S>>) -> String
    where S: Into<String> {
        let mut labelstring = String::new();
        if let Some(labellist) = labellist {
            if ! labellist.is_empty() {
                for label in labellist {
                    if labelstring.is_empty() {
                        labelstring = format!("o={}", label.into());
                    } else {
                        labelstring = format!("{}&o={}", labelstring, label.into());
                    }
                }
            }
        };

        debug!("label-string: '{}'", labelstring);
        labelstring
    }

    /// generic helper function for calling of call object
    ///
    /// The `desc` parameter is a short description for  error messages, its embedded into 'Problem
    /// '...' with <DESC>'.
    /// The call is executed with the `path` parameter and the `httpmethod` with `uploaddata` for
    /// `Put` and `Post` http methods.
    fn execute<INPUT,OUTPUT>(c: &Changes, desc: &str, path: &str, httpmethod: call::CallMethod, uploaddata: Option<&INPUT>) -> GGRResult<OUTPUT>
    where INPUT: serde::Serialize + std::fmt::Debug,
          OUTPUT: serde::de::DeserializeOwned
    {
        match c.call.request(httpmethod, path, uploaddata) {
            Ok(cr) => {
                match cr.status() {
                    200 | 201 | 202 | 203 | 205 => cr.convert::<OUTPUT>(),
                    /*
                     * We need handling of 204 returne code. 204 means no body text what we can
                     * convert. But the converter try it and crashes with a json failure
                     * "JsonError(ErrorImpl { code: EofWhileParsingValue, line: 1, column: 0 })"
                     *
                     * 204 => cr.convert::<OUTPUT>(),
                     */
                    status => { Err(GGRError::GerritApiError(GerritError::GerritApi(status, String::from_utf8(cr.get_body().unwrap_or_else(|| "no cause from server".into()))?))) },
                }
            },
            Err(x) => {
                Err(GGRError::General(format!("Problem '{}' with {}", x, desc)))
            }
        }
    }

    /// api function 'GET /changes/'
    pub fn query_changes<S>(&mut self, querylist: Option<Vec<S>>, labellist: Option<Vec<S>>) -> GGRResult<Vec<entities::ChangeInfo>>
    where S: Into<String> {
        let mut querystring = format!("pp=0{}", Changes::build_query_string(querylist));
        let labelstring = Changes::build_label_string(labellist);
        if ! labelstring.is_empty() {
            querystring = format!("{}&{}", querystring, labelstring);
        }

        querystring = querystring.replace("//", "/");
        querystring = querystring.replace("//", "/");
        querystring = querystring.replace("//", "/");

        self.call.set_url_query(Some(&querystring));

        let path = format!("{}/", ENDPOINT);

        Changes::execute::<(),Vec<entities::ChangeInfo>>(self, "query change", &path, call::CallMethod::Get, None)
    }

    /// api function 'POST /changes'
    ///
    /// V02.10
    pub fn create_change(&self, ci: &entities::ChangeInput) -> GGRResult<entities::ChangeInfo> {
        if ci.project.is_empty() || ci.branch.is_empty() || ci.subject.is_empty() {
            return Err(GGRError::GerritApiError(GerritError::ChangeInputProblem));
        }

        let config = config::Config::new(self.call.get_base());
        if let Err(x) = config.check_version("POST /changes/".into(), "2.10.0".into()) {
            return Err(x);
        }

        Changes::execute(self, "change create", ENDPOINT, call::CallMethod::Post, Some(&ci))
    }

    /// api function 'GET /changes/{change-id}'
    pub fn get_change(&mut self, changeid: &str, features: Option<Vec<&str>>) -> GGRResult<entities::ChangeInfo> {
        if changeid.is_empty() {
            return Err(GGRError::GerritApiError(GerritError::ChangeIDEmpty));
        }

        let query = Changes::build_label_string(features);

        let path = format!("{}/{}", ENDPOINT, changeid);

        self.call.set_url_query(Some(&query));

        Changes::execute::<(),entities::ChangeInfo>(self, "get change", &path, call::CallMethod::Get, None)
    }

    /// api function 'GET /changes/{change-id}/detail'
    pub fn get_change_detail(&self, changeid: &str) -> GGRResult<entities::ChangeInfo> {
        if changeid.is_empty() {
            return Err(GGRError::GerritApiError(GerritError::ChangeIDEmpty));
        }

        let path = format!("{}/{}/detail", ENDPOINT, changeid);

        Changes::execute::<(),entities::ChangeInfo>(self, "get change detail", &path, call::CallMethod::Get, None)
    }

    /// api function `GET /changes/{change-id}/reviewers/'
    pub fn get_reviewers(&self, changeid: &str) -> GGRResult<Vec<entities::ReviewerInfo>> {
        if changeid.is_empty() {
            return Err(GGRError::GerritApiError(GerritError::ChangeIDEmpty));
        }

        let path = format!("{}/{}/reviewers/", ENDPOINT, changeid);

        Changes::execute::<(),Vec<entities::ReviewerInfo>>(self, "receiving reviewer list", &path, call::CallMethod::Get, None)
    }

    /// api function 'POST /changes/{change-id}/reviewers'
    pub fn add_reviewer(&self, changeid: &str, reviewer: &str) -> GGRResult<entities::AddReviewerResult> {
        if changeid.is_empty() || reviewer.is_empty() {
            return Err(GGRError::GerritApiError(GerritError::GetReviewerListProblem("changeid or reviewer is empty".into())));
        }

        let path = format!("{}/{}/reviewers", ENDPOINT, changeid);

        let reviewerinput = entities::ReviewerInput {
                reviewer: reviewer.into(),
                confirmed: None,
                state: None,
        };

        Changes::execute::<&entities::ReviewerInput,entities::AddReviewerResult>(self, "add reviewer", &path, call::CallMethod::Post, Some(&&reviewerinput))
    }

    /// api function 'DELETE /changes/{change-id}/reviewers/{account-id}'
    pub fn delete_reviewer(&self, changeid: &str, reviewer: &str) -> GGRResult<()> {
        if changeid.is_empty() || reviewer.is_empty() {
            return Err(GGRError::GerritApiError(GerritError::GetReviewerListProblem("changeid or reviewer is empty".into())));
        }

        let path = format!("{}/{}/reviewers/{}", ENDPOINT, changeid, reviewer);

        Changes::execute::<(),()>(self, "deleting reviewer", &path, call::CallMethod::Delete, None)
    }

    /// api function 'POST /changes/{change-id}/abandon'
    ///
    /// notify is one of `none`, `owner`, `owner_reviewers` or `all`.
    pub fn abandon_change(&self, changeid: &str, message: Option<&str>, notify: Option<&str>) -> GGRResult<entities::ChangeInfo> {
        if changeid.is_empty() {
            return Err(GGRError::GerritApiError(GerritError::ChangeIDEmpty));
        }

        let path = format!("{}/{}/abandon", ENDPOINT, changeid);

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

        let path = format!("{}/{}/restore", ENDPOINT, changeid);

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

        let path = format!("{}/{}/revisions/{}/review", ENDPOINT, changeid, revisionid);

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
