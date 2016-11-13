
//! All entities of a gerrit instance
//!
//! The entities are documented on gerrit site on
//! <https://gerrit-documentation.storage.googleapis.com/Documentation/2.12.3/rest-api-changes.html#json-entities>.
//!
//! **NOTICE**: Only current needed entities are here reflected.

use std::collections::HashMap;


#[derive(Deserialize, Debug, Clone)]
pub struct AccountInfo {
    pub _account_id: Option<u64>,
    pub name: Option<String>,
    pub email: Option<String>,
    pub secondary_emails: Option<Vec<String>>,
    pub username: Option<String>,
    pub _more_accounts: Option<String>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ActionInfo {
    pub method: Option<String>,
    pub label: Option<String>,
    pub title: Option<String>,
    pub enabled: Option<String>,
}

// the enum variants must be in upper case letters, the server will send them in this style
#[derive(Deserialize, Debug, Clone)]
pub enum ChangeStatus {
    NEW,
    MERGED,
    ABANDONED,
    DRAFT,
}

#[derive(Deserialize, Debug, Clone)]
pub struct LabelInfo {
    pub optional: Option<bool>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ChangeMessageInfo {
    pub id: String,
    pub author: Option<AccountInfo>,
    pub date: String,
    pub message: String,
    pub tag: Option<String>,
    pub _revision_number: Option<u16>,
}

#[allow(non_camel_case_types)]
#[derive(Deserialize, Debug, Clone)]
pub enum RevisionInfoKind {
    REWORK,
    TRIVIAL_REBASE,
    MERGE_FIRST_PARENT_UPDATE,
    NO_CODE_CHANGE,
    NO_CHANGE,
}

#[derive(Deserialize, Debug, Clone)]
pub struct FetchInfo {
    pub url: String,
    #[serde(rename="ref")] // "ref" is a keyword
    pub reference: String,
    pub commands: Option<String>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct GitPersonInfo {
    pub name: String,
    pub email: String,
    pub date: String,
    pub tz: u16,
}

#[derive(Deserialize, Debug, Clone)]
pub struct CommitInfoParents {
    pub commit: String,
    pub author: Option<GitPersonInfo>,
    pub committer: Option<GitPersonInfo>,
    pub subject: String,
    pub message: Option<String>,
    pub web_links: Option<String>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct CommitInfo {
    pub commit: Option<String>,
    pub parents: Option<Vec<CommitInfoParents>>,
    pub author: Option<GitPersonInfo>,
    pub committer: Option<GitPersonInfo>,
    pub subject: Option<String>,
    pub message: Option<String>,
    pub web_links: Option<String>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct RevisionInfo {
    pub draft: Option<bool>,
    pub has_draft_comments: Option<bool>,
    pub _number: u64,
    pub created: Option<String>,
    pub uploader: Option<AccountInfo>,
    #[serde(rename="ref")] // "ref" is a keyword
    pub reference: Option<String>,
    pub fetch: HashMap<String, FetchInfo>,
    pub commit: Option<CommitInfo>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ProblemInfo {
    pub message: String,
    pub status: Option<String>,
    pub outcome: Option<String>,
}

#[derive(Deserialize, Debug, Clone)]
pub enum ProjectState {
    ACTIVE,
    READONLY,
    HIDDEN,
}

#[derive(Deserialize, Debug, Clone)]
pub enum ProjectTypes {
    ALL,
    CODE,
    PERMISSIONS,
}

#[derive(Deserialize, Debug, Clone)]
pub struct WebLinkInfo {
    pub name: String,
    pub url: String,
    pub image_url: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ProjectInfo {
    pub name: Option<String>,
    pub id: String,
    pub parent: Option<String>,
    pub description: Option<String>,
    pub state: Option<ProjectState>,
    pub branches: Option<HashMap<String, String>>,
    pub web_links: Option<Vec<WebLinkInfo>>,
}

#[derive(Deserialize, Debug, Clone)]
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
    pub _number: u64,
    pub owner: AccountInfo,
    pub action: Option<Vec<ActionInfo>>,
    pub labels: Option<HashMap<String, LabelInfo>>,
    pub permitted_labels: Option<HashMap<String, LabelInfo>>,
    pub removeable_reviewers: Option<Vec<AccountInfo>>,
    pub reviewers: Option<String>,
    pub messages: Option<HashMap<String, ChangeMessageInfo>>,
    pub current_revision: Option<String>,
    pub revision: Option<HashMap<String, RevisionInfo>>,
    pub revisions: Option<HashMap<String, RevisionInfo>>,
    pub _more_changes: Option<bool>,
    pub problems: Option<Vec<ProblemInfo>>,
}
