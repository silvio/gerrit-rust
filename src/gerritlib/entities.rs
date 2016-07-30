
//! All entities of a gerrit instance
//!
//! The entities are documented on gerrit site on
//! <https://gerrit-documentation.storage.googleapis.com/Documentation/2.12.3/rest-api-changes.html#json-entities>.
//!
//! **NOTICE**: Only current needed entities are here reflected.

use std::collections::HashMap;
use std::fmt;
use std::fmt::Display;

#[derive(RustcDecodable, Debug)]
pub struct AccountInfo {
    pub _account_id: u64,
    pub name: Option<String>,
    pub email: Option<String>,
    pub secondary_emails: Option<Vec<String>>,
    pub username: Option<String>,
    pub _more_accounts: Option<String>,
}

impl Display for AccountInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(x) = self.name.clone() {
            try!(write!(f, "{}", x));
        } else {
            try!(write!(f, "uid:{}", self._account_id));
        };

        if let Some(x) = self.email.clone() {
            try!(write!(f, "{}", x));
        };

        Ok(())
    }
}

#[derive(RustcDecodable, Debug)]
pub struct ActionInfo {
    pub method: Option<String>,
    pub label: Option<String>,
    pub title: Option<String>,
    pub enabled: Option<String>,
}

// the enum variants must be in upper case letters, the server will send them in this style
#[derive(RustcDecodable, Debug)]
pub enum ChangeStatus {
    NEW,
    MERGED,
    ABANDONED,
    DRAFT,
}

impl Display for ChangeStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match *self {
            ChangeStatus::NEW => "NEW",
            ChangeStatus::MERGED => "MERGED",
            ChangeStatus::ABANDONED => "ABANDONED",
            ChangeStatus::DRAFT => "DRAFT",
        };
        write!(f, "{}", s)
    }
}

#[derive(RustcDecodable, Debug)]
pub struct LabelInfo {
    pub optional: Option<bool>,
}

#[derive(RustcDecodable, Debug)]
pub struct ChangeMessageInfo {
    pub id: String,
    pub author: Option<AccountInfo>,
    pub date: String,
    pub message: String,
    pub tag: Option<String>,
    pub _revision_number: Option<u16>,
}

#[allow(non_camel_case_types)]
#[derive(RustcDecodable, Debug)]
pub enum RevisionInfoKind {
    REWORK,
    TRIVIAL_REBASE,
    MERGE_FIRST_PARENT_UPDATE,
    NO_CODE_CHANGE,
    NO_CHANGE,
}

#[derive(RustcDecodable, Debug)]
pub struct FetchInfo {
    pub url: String,
    pub reference: String,
    pub commands: Option<String>,
}

#[derive(RustcDecodable, Debug)]
pub struct RevisionInfo {
    pub draft: bool,
    pub kind: RevisionInfoKind,
    pub _number: u16,
    pub created: String,
    pub uploader: AccountInfo,
    pub reference: String,
    pub fetch: HashMap<String, FetchInfo>,
    /* TODO: Some fileds ommited */
}

#[derive(RustcDecodable, Debug)]
pub struct ProblemInfo {
    pub message: String,
    pub status: Option<String>,
    pub outcome: Option<String>,
}

#[derive(RustcDecodable, Debug)]
pub struct ChangeInfo {
    pub id: String,
    pub project: String,
    pub branch: String,
    pub topic: Option<String>,
    pub change_id: String,
    pub subject: String,
    pub status: ChangeStatus,
    pub created: String,
    pub updated: String,
    pub submitted: Option<String>,
    pub starred: Option<bool>,
    pub stars: Option<Vec<String>>,
    pub reviewed: Option<bool>,
    pub submit_type: Option<String>,
    pub mergeable: Option<bool>,
    pub insertions: u16,
    pub deletions: u16,
    pub _number: u16,
    pub owner: AccountInfo,
    pub action: Option<Vec<ActionInfo>>,
    pub labels: Option<HashMap<String, LabelInfo>>,
    pub permitted_labels: Option<HashMap<String, LabelInfo>>,
    pub removeable_reviewers: Option<Vec<AccountInfo>>,
    pub reviewers: Option<String>,
    pub messages: Option<HashMap<String, ChangeMessageInfo>>,
    pub current_revision: Option<HashMap<String, RevisionInfo>>,
    pub revision: Option<HashMap<String, RevisionInfo>>,
    pub _more_changes: Option<bool>,
    pub problems: Option<Vec<ProblemInfo>>,
}

/// **TODO**: This is subject to change! Its to unflexible.
impl Display for ChangeInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        try!(writeln!(f, "* {project:.<41.41} {branch:.<20.20} {subject:.>30.30}", project=self.project, branch=self.branch, subject=self.subject));
        try!(writeln!(f, "| {changeid:41.41} {number:<31.10} {created:>19.19}", changeid=self.change_id, number=self._number, created=self.created));
        try!(writeln!(f, "| {topic:73.41} {updated:19.19}", topic=self.topic.clone().unwrap_or("N/A".to_string()), updated=self.updated));
        write!(f, "` {status}, {owner}", owner=self.owner, status=self.status)
    }
}

#[derive(RustcDecodable, Debug)]
pub enum ProjectState {
    ACTIVE,
    READONLY,
    HIDDEN,
}

#[derive(RustcDecodable, Debug, Clone)]
pub enum ProjectTypes {
    ALL,
    CODE,
    PERMISSIONS,
}

#[derive(RustcDecodable, Debug)]
pub struct WebLinkInfo {
    pub name: String,
    pub url: String,
    pub image_url: String,
}

#[derive(RustcDecodable, Debug)]
pub struct ProjectInfo {
    pub name: Option<String>,
    pub id: String,
    pub parent: Option<String>,
    pub description: Option<String>,
    pub state: Option<ProjectState>,
    pub branches: Option<HashMap<String, String>>,
    pub web_links: Option<Vec<WebLinkInfo>>,
}
