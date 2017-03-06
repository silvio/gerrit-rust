
//! All entities of a gerrit instance
//!
//! The entities are documented on gerrit site on
//! <https://gerrit-documentation.storage.googleapis.com/Documentation/2.12.3/rest-api-changes.html#json-entities>.
//!
//! **NOTICE**: Only current needed entities are here reflected.

#![warn(missing_docs)]

use std::collections::HashMap;

/// The `AccountInfo0209` entity contains information about an account
#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct AccountInfo0209 {
    /// The numeric ID of the account
    pub _account_id: Option<u64>,
    /// The full name of the user.
    /// Only set if detailed account information is requested
    pub name: Option<String>,
    /// The email address the user prefers to be contacted through.
    /// Only set if detailed account information is requested
    pub email: Option<String>,
    /// The username of the user.
    /// Only set if detailed account information is requested
    pub username: Option<String>,
}

/// The `AccountInfo0213` entity contains information about an account
#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct AccountInfo0213 {
    /// The numeric ID of the account
    pub _account_id: Option<u64>,
    /// The full name of the user. Only set if detailed account information is requested. See
    /// option DETAILED_ACCOUNTS for change queries and option DETAILS for account queries.  
    /// (optional)
    pub name: Option<String>,
    /// The email address the user prefers to be contacted through. Only set if detailed account
    /// information is requested. See option DETAILED_ACCOUNTS for change queries and options
    /// DETAILS and ALL_EMAILS for account queries.  
    /// (optional)
    pub email: Option<String>,
    /// A list of the secondary email addresses of the user. Only set for account queries when the
    /// ALL_EMAILS option is set.  
    /// (optional)
    pub secondary_emails: Option<Vec<String>>,
    /// The username of the user. Only set if detailed account information is requested. See option
    /// DETAILED_ACCOUNTS for change queries and option DETAILS for account queries.  
    /// (optional)
    pub username: Option<String>,
    /// Whether the query would deliver more results if not limited. Only set on the last account
    /// that is returned.  
    /// (optional, not set if false)
    pub _more_accounts: Option<String>,
}

/// `AccountInfo` differs between Gerrit server/protocoll versions. This enum hold them together.
#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(untagged)]
pub enum AccountInfo {
    /// V2.09
    Gerrit0209(AccountInfo0209),
    /// V2.13
    Gerrit0213(AccountInfo0213),
}

/// The `ActionInfo` entity describes a REST API call the client can make to manipulate a resource.
/// These are frequently implemented by plugins and may be discovered at runtime.
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ActionInfo {
    /// HTTP method to use with the action. Most actions use POST, PUT or DELETE to cause state
    /// changes.  
    /// (optional)
    pub method: Option<String>,
    /// Short title to display to a user describing the action. In the Gerrit web interface the
    /// label is used as the text on the button presented in the UI.  
    /// (optional)
    pub label: Option<String>,
    /// Longer text to display describing the action. In a web UI this should be the title
    /// attribute of the element, displaying when the user hovers the mouse.  
    /// (optional)
    pub title: Option<String>,
    /// If true the action is permitted at this time and the caller is likely allowed to execute
    /// it. This may change if state is updated at the server or permissions are modified. Not
    /// present if false.  
    /// (optional)
    pub enabled: Option<String>,
}

/// `ChangeInfo` helper variant to present a status of a change
#[allow(non_camel_case_types)]
#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum ChangeInfo_ChangeStatus {
    /// new change
    NEW,
    /// change is merged
    MERGED,
    /// change is abandoned
    ABANDONED,
    /// its a draft change
    DRAFT,
}

/// The `LabelInfo` entity contains information about a label on a change, always corresponding to
/// the current patch set.
///
/// There are two options that control the contents of `LabelInfo`: `LABELS` and `DETAILED_LABELS`.
///
/// * For a quick summary of the state of labels, use `LABELS`.
/// * For detailed information about labels, including exact numeric votes for all users and the
///   allowed range of votes for the current user, use `DETAILED_LABELS`.
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct LabelInfo {
    /// Whether the label is optional. Optional means the label may be set, but it’s neither
    /// necessary for submission nor does it block submission if set.  
    /// (optional)
    pub optional: Option<bool>,
}

/// The `ChangeMessageInfo` entity contains information about a message attached to a change.
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ChangeMessageInfo {
    /// The ID of the message.
    pub id: String,
    /// Author of the message as an AccountInfo entity. Unset if written by the Gerrit system.  
    /// (optional)
    pub author: Option<AccountInfo>,
    /// The timestamp this message was posted.
    pub date: String,
    /// The text left by the user.
    pub message: String,
    /// Value of the tag field from ReviewInput set while posting the review. NOTE: To apply
    /// different tags on on different votes/comments multiple invocations of the REST call are
    /// required.  
    /// (optional)
    pub tag: Option<String>,
    /// Which patchset (if any) generated this message.  
    /// (optional)
    pub _revision_number: Option<u16>,
}

/// The `FetchInfo` entity contains information about how to fetch a patch set via a certain
/// protocol.
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct FetchInfo {
    /// The URL of the project.
    pub url: String,
    /// The ref of the patch set.
    #[serde(rename="ref")] // "ref" is a keyword
    pub reference: String,
    /// The download commands for this patch set as a map that maps the command names to the
    /// commands.
    /// Only set if download commands are requested.  
    /// (optional)
    pub commands: Option<String>,
}

/// The `GitPersonInfo` entity contains information about the author/committer of a commit.
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct GitPersonInfo {
    /// The name of the author/committer.
    pub name: String,
    /// The email address of the author/committer.
    pub email: String,
    /// The timestamp of when this identity was constructed.
    pub date: String,
    /// The timezone offset from UTC of when this identity was constructed.
    pub tz: u16,
}

/// `CommitInfoParents`, same as `CommitInfo` but commit is string
#[allow(missing_docs)]
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct CommitInfoParents {
    pub commit: String,
    pub author: Option<GitPersonInfo>,
    pub committer: Option<GitPersonInfo>,
    pub subject: String,
    pub message: Option<String>,
    pub web_links: Option<String>,
}

/// The `CommitInfo` entity contains information about a commit.
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct CommitInfo {
    /// The commit ID. Not set if included in a RevisionInfo entity that is contained in a map
    /// which has the commit ID as key.  
    /// (optional)
    pub commit: Option<String>,
    /// The parent commits of this commit as a list of CommitInfo entities. In each parent only the
    /// commit and subject fields are populated.  
    /// (optional)
    pub parents: Option<Vec<CommitInfoParents>>,
    /// The author of the commit as a GitPersonInfo entity.
    pub author: Option<GitPersonInfo>,
    /// The committer of the commit as a GitPersonInfo entity.
    pub committer: Option<GitPersonInfo>,
    /// The subject of the commit (header line of the commit message).
    pub subject: Option<String>,
    /// The commit message.
    pub message: Option<String>,
    /// Links to the commit in external sites as a list of WebLinkInfo entities.  
    /// (optional)
    pub web_links: Option<String>,
}

/// The `FileInfo` entity contains information about a file in a patch set.
#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct FileInfo0209 {
    /// The status of the file ("A"=Added, "D"=Deleted, "R"=Renamed, "C"=Copied, "W"=Rewritten).
    /// Not set if the file was Modified ("M").
    pub status: Option<String>,
    /// Whether the file is binary.
    pub binary: Option<bool>,
    /// The old file path.
    /// Only set if the file was renamed or copied.
    pub old_path: Option<String>,
    /// Number of inserted lines.
    /// Not set for binary files or if no lines were inserted.
    pub lines_inserted: Option<u64>,
    /// Number of deleted lines.
    /// Not set for binary files or if no lines were deleted.
    pub lines_deleted: Option<String>,
}

/// The `FileInfo` entity contains information about a file in a patch set.
#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct FileInfo0213 {
    /// The status of the file ("A"=Added, "D"=Deleted, "R"=Renamed, "C"=Copied, "W"=Rewritten).
    /// Not set if the file was Modified ("M").
    pub status: Option<String>,
    /// Whether the file is binary.
    pub binary: Option<bool>,
    /// The old file path.
    /// Only set if the file was renamed or copied.
    pub old_path: Option<String>,
    /// Number of inserted lines.
    /// Not set for binary files or if no lines were inserted.
    pub lines_inserted: Option<u64>,
    /// Number of deleted lines.
    /// Not set for binary files or if no lines were deleted.
    pub lines_deleted: Option<String>,
    /// Number of bytes by which the file size increased/decreased.
    pub size_delta: u64,
    /// File size in bytes.
    pub size: u64,
}

/// `FileInfo` differs between Gerrit server/protocoll versions. This enum hold them together.
#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(untagged)]
pub enum FileInfo {
    /// V2.09
    Gerrit0209(FileInfo0209),
    /// V2.13
    Gerrit0213(FileInfo0213),
}


/// The `RevisionInfo` entity contains information about a patch set.
#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct RevisionInfo0209 {
    /// Whether the patch set is a draft.
    pub draft: Option<bool>,
    /// Whether the patch set has one or more draft comments by the calling user. Only set if draft
    /// comments is requested.
    pub has_draft_comments: Option<bool>,
    /// The patch set number.
    pub _number: u64,
    /// Information about how to fetch this patch set. The fetch information is provided as a map
    /// that maps the protocol name ("git", "http", "ssh") to FetchInfo entities.
    pub fetch: HashMap<String, FetchInfo>,
    /// The commit of the patch set as `CommitInfo` entity.
    pub commit: Option<CommitInfo>,
    /// The files of the patch set as a map that maps the file names to `FileInfo` entities.
    pub files: Option<HashMap<String, FileInfo0209>>,
    /// Actions the caller might be able to perform on this revision. The information is a map of
    /// view name to ActionInfo entities.
    pub actions: Option<HashMap<String, ActionInfo>>,
}

#[allow(non_camel_case_types)]
#[allow(missing_docs)]
#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum RevisionInfo0213_ChangeKind {
    #[allow(missing_docs)]
    REWORK,
    #[allow(missing_docs)]
    TRIVIAL_REBASE,
    #[allow(missing_docs)]
    MERGE_FIRST_PARENT_UPDATE,
    #[allow(missing_docs)]
    NO_CODE_CHANGE,
    #[allow(missing_docs)]
    NO_CHANGE,
}

/// The `RevisionInfo` entity contains information about a patch set. Not all fields are returned by
/// default. Additional fields can be obtained by adding o parameters as described in Query
/// Changes.
#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct RevisionInfo0213 {
    /// Whether the patch set is a draft.  
    /// (optional)
    pub draft: Option<bool>,
    /// The change kind. Valid values are REWORK, TRIVIAL_REBASE, MERGE_FIRST_PARENT_UPDATE,
    /// NO_CODE_CHANGE, and NO_CHANGE.
    pub kind: RevisionInfo0213_ChangeKind,
    /// The patch set number.
    pub _number: u64,
    /// The timestamp of when the patch set was created.  
    /// (v2.15)
    pub created: String,
    /// The uploader of the patch set as an AccountInfo entity.
    pub uploader: AccountInfo,
    /// The Git reference for the patch set.
    #[serde(rename="ref")] // "ref" is a keyword
    pub reference: String,
    /// Information about how to fetch this patch set. The fetch information is provided as a map
    /// that maps the protocol name (“git”, “http”, “ssh”) to FetchInfo entities. This information
    /// is only included if a plugin implementing the download commands interface is installed.
    pub fetch: HashMap<String, FetchInfo>,
    /// The commit of the patch set as `CommitInfo` entity.
    pub commit: Option<CommitInfo>,
    /// The files of the patch set as a map that maps the file names to FileInfo entities. Only set
    /// if CURRENT_FILES or ALL_FILES option is requested.
    pub files: Option<FileInfo0213>,
    /// Actions the caller might be able to perform on this revision. The information is a map of
    /// view name to ActionInfo entities.
    pub actions: Option<HashMap<String, ActionInfo>>,
    /// Indicates whether the caller is authenticated and has commented on the current revision.
    /// Only set if REVIEWED option is requested.
    pub reviewed: Option<bool>,
    /// If the COMMIT_FOOTERS option is requested and this is the current patch set, contains the
    /// full commit message with Gerrit-specific commit footers, as if this revision were submitted
    /// using the Cherry Pick submit type.
    #[serde(rename="messageWithFooter")] // "ref" is a keyword
    pub message_with_footer: Option<String>,
    /// If the PUSH_CERTIFICATES option is requested, contains the push certificate provided by the
    /// user when uploading this patch set as a PushCertificateInfo entity. This field is always
    /// set if the option is requested; if no push certificate was provided, it is set to an empty
    /// object.
    pub push_certificate: Option<PushCertificateInfo>,
}

/// `RevisionInfo` differs between Gerrit server/protocoll versions. This enum hold them together.
#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(untagged)]
pub enum RevisionInfo {
    /// V2.09
    Gerrit0209(Box<RevisionInfo0209>),
    /// V2.13
    Gerrit0213(Box<RevisionInfo0213>),
}

/// The `PushCertificateInfo` entity contains information about a push certificate provided when
/// the user pushed for review with git push --signed HEAD:refs/for/<branch>. Only used when signed
/// push is enabled on the server.
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct PushCertificateInfo {
    /// Signed certificate payload and GPG signature block.
    pub certificate: String,
    /// Information about the key that signed the push, along with any problems found while
    /// checking the signature or the key itself, as a GpgKeyInfo entity.
    pub key: GpgKeyInfo,
}

/// The `GpgKeyInfo` entity contains information about a GPG public key.
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct GpgKeyInfo {
    /// The 8-char hex GPG key ID.
    pub id: Option<String>,
    /// The 40-char (plus spaces) hex GPG key fingerprint.
    pub fingerprint: Option<String>,
    /// OpenPGP User IDs associated with the public key.
    pub user_ids: String,
    /// ASCII armored public key material.
    pub key: String,
    /// The result of server-side checks on the key; one of BAD, OK, or TRUSTED. BAD keys have
    /// serious problems and should not be used. If a key is OK, inspecting only that key found no
    /// problems, but the system does not fully trust the key’s origin. A `TRUSTED key is valid,
    /// and the system knows enough about the key and its origin to trust it.
    pub status: Option<String>,
    /// A list of human-readable problem strings found in the course of checking whether the key is
    /// valid and trusted.
    pub problems: Option<String>,
}


/// The `ProblemInfo` entity contains a description of a potential consistency problem with a change.
/// These are not related to the code review process, but rather indicate some inconsistency in
/// Gerrit’s database or repository metadata related to the enclosing change.
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ProblemInfo {
    /// Plaintext message describing the problem with the change.
    pub message: String,
    /// The status of fixing the problem (FIXED, FIX_FAILED). Only set if a fix was attempted.  
    /// (optional)
    pub status: Option<String>,
    /// If status is set, an additional plaintext message describing the outcome of the fix.  
    /// (optional)
    pub outcome: Option<String>,
}

/// `ProjectInfo` helper variant to present a status of a project
#[allow(non_camel_case_types)]
#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum ProjectInfo_ProjectState {
    /// The project is active
    ACTIVE,
    /// Project is read only, noch anges possible
    READONLY,
    /// project is hidden
    HIDDEN,
}

/// The `WebLinkInfo` entity describes a link to an external site.
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct WebLinkInfo {
    /// The link name.
    pub name: String,
    /// The link URL.
    pub url: String,
    /// URL to the icon of the link.
    pub image_url: String,
}

/// The `ProjectInfo` entity contains information about a project.
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ProjectInfo {
    /// The name of the project.  
    /// (optional, not set if returned in a map where the project name is used as map key)
    pub name: Option<String>,
    /// The URL encoded project name.
    pub id: String,
    /// The name of the parent project.
    /// ?-<n> if the parent project is not visible (<n> is a number which is increased for each
    /// non-visible project).  
    /// (optional)
    pub parent: Option<String>,
    /// The description of the project.  
    /// (optional)
    pub description: Option<String>,
    /// ACTIVE, READ_ONLY or HIDDEN.  
    /// (optional)
    pub state: Option<ProjectInfo_ProjectState>,
    /// Map of branch names to HEAD revisions.  
    /// (optional)
    pub branches: Option<HashMap<String, String>>,
    /// Links to the project in external sites as a list of WebLinkInfo entries.  
    /// (optional, 2.15)
    pub web_links: Option<Vec<WebLinkInfo>>,
}

/// The `ChangeInfo` entity contains information about a change.
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ChangeInfo {
    /// The ID of the change in the format "'<project>~<branch>~<Change-Id>'", where 'project',
    /// 'branch' and 'Change-Id' are URL encoded. For 'branch' the refs/heads/ prefix is omitted.
    pub id: String,
    /// The name of the project.
    pub project: String,
    /// The name of the target branch.
    /// The refs/heads/ prefix is omitted.
    pub branch: String,
    /// The topic to which this change belongs.  
    /// (optional)
    pub topic: Option<String>,
    /// The Change-Id of the change.
    pub change_id: String,
    /// The subject of the change (header line of the commit message).
    pub subject: String,
    /// The status of the change (NEW, MERGED, ABANDONED, DRAFT).
    pub status: ChangeInfo_ChangeStatus,
    /// The timestamp of when the change was created.
    pub created: String,
    /// The timestamp of when the change was last updated.
    pub updated: String,
    /// The timestamp of when the change was submitted.  
    /// (optional, only set for merged changes)
    pub submitted: Option<String>,
    /// Whether the calling user has starred this change with the default label.  
    /// (optional)
    pub starred: Option<bool>,
    /// A list of star labels that are applied by the calling user to this change. The labels are
    /// lexicographically sorted.  
    /// (optional)
    pub stars: Option<Vec<String>>,
    /// Whether the change was reviewed by the calling user. Only set if reviewed is requested.  
    /// (optional)
    pub reviewed: Option<bool>,
    /// The submit type of the change.
    /// Not set for merged changes.  
    /// (optional)
    pub submit_type: Option<String>,
    /// Whether the change is mergeable.
    /// Not set for merged changes, or if the change has not yet been tested.  
    /// (optional)
    pub mergeable: Option<bool>,
    /// Number of inserted lines.
    pub insertions: u16,
    /// Number of deleted lines.
    pub deletions: u16,
    /// The legacy numeric ID of the change.
    pub _number: u64,
    /// The owner of the change as an AccountInfo entity.
    pub owner: AccountInfo,
    /// Actions the caller might be able to perform on this revision. The information is a map of
    /// view name to ActionInfo entities.  
    /// (optional)
    pub action: Option<Vec<ActionInfo>>,
    /// The labels of the change as a map that maps the label names to LabelInfo entries.
    /// Only set if labels or detailed labels are requested.  
    /// (optional)
    pub labels: Option<HashMap<String, LabelInfo>>,
    /// A map of the permitted labels that maps a label name to the list of values that are allowed
    /// for that label.
    /// Only set if detailed labels are requested.  
    /// (optional)
    pub permitted_labels: Option<HashMap<String, LabelInfo>>,
    /// The reviewers that can be removed by the calling user as a list of AccountInfo entities.
    /// Only set if detailed labels are requested.  
    /// (optional)
    pub removeable_reviewers: Option<Vec<AccountInfo>>,
    /// The reviewers as a map that maps a reviewer state to a list of AccountInfo entities.
    /// Possible reviewer states are REVIEWER, CC and REMOVED.
    /// REVIEWER: Users with at least one non-zero vote on the change.
    /// CC: Users that were added to the change, but have not voted.
    /// REMOVED: Users that were previously reviewers on the change, but have been removed.
    /// Only set if detailed labels are requested.  
    /// (optional)
    // TODO: own enum
    pub reviewers: Option<String>,
    /// Messages associated with the change as a list of ChangeMessageInfo entities.
    /// Only set if messages are requested.  
    /// (optional)
    pub messages: Option<HashMap<String, ChangeMessageInfo>>,
    /// The commit ID of the current patch set of this change.
    /// Only set if the current revision is requested or if all revisions are requested.  
    /// (optional)
    pub current_revision: Option<String>,
    /// All patch sets of this change as a map that maps the commit ID of the patch set to a
    /// RevisionInfo entity.
    /// Only set if the current revision is requested (in which case it will only contain a key for
    /// the current revision) or if all revisions are requested.  
    /// (optional)
    pub revisions: Option<HashMap<String, RevisionInfo>>,
    /// Whether the query would deliver more results if not limited.
    /// Only set on the last change that is returned.  
    /// (optional)
    pub _more_changes: Option<bool>,
    /// A list of ProblemInfo entities describing potential problems with this change. Only set if
    /// CHECK is set.  
    /// (optional)
    pub problems: Option<Vec<ProblemInfo>>,
}

/// The `ChangeInput` entity contains information about creating a new change.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChangeInput {
    /// The name of the project.
    pub project: String,
    /// The name of the target branch.
    /// The refs/heads/ prefix is omitted.
    pub branch: String,
    /// The subject of the change (header line of the commit message).
    pub subject: String,
    /// The topic to which this change belongs.  
    /// (optional)
    pub topic: Option<String>,
    /// The status of the change (only NEW and DRAFT accepted here).  
    /// (optional)
    // TODO: Only NEW and DRAFT allowed
    pub status: Option<String>,
    /// A {change-id} that identifies the base change for a create change operation.  
    /// (optional)
    pub base_change: Option<String>,
    /// Allow creating a new branch when set to true.  
    /// (optional)
    pub new_branch: Option<bool>,
    /// The detail of a merge commit as a MergeInput entity.  
    /// (optiional)
    pub merge: Option<MergeInput>,
}

/// The `MergeInput` entity contains information about the merge
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MergeInput {
    /// The source to merge from, e.g. a complete or abbreviated commit SHA-1, a complete reference
    /// name, a short reference name under refs/heads, refs/tags, or refs/remotes namespace, etc.
    pub source: String,
    /// The strategy of the merge, can be recursive, resolve, simple-two-way-in-core, ours or
    /// theirs, default will use project settings.  
    /// (optional)
    // TODO: only recursive, resolve, simple-two-way-in-core, ours or theirs allowed
    pub strategy: Option<String>,
}

/// The `ReviewerInfo0209` entity contains information about a reviewer and its votes on a change.
///
/// `ReviewerInfo0209` has the same fields as AccountInfo and includes detailed account
/// information.
#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct ReviewerInfo0209 {
    /// The numeric ID of the account
    pub _account_id: Option<u64>,
    /// The full name of the user.
    /// Only set if detailed account information is requested
    pub name: Option<String>,
    /// The email address the user prefers to be contacted through.
    /// Only set if detailed account information is requested
    pub email: Option<String>,
    /// The username of the user.
    /// Only set if detailed account information is requested
    pub username: Option<String>,
    /// gerritcodereview#reviewer
    kind: String,
    /// The approvals of the reviewer as a map that maps the label names to the approval values
    /// ("-2", "-1", "0", "+1", "+2")
    pub approvals: ReviewerInfoApprovals,
}

/// The `ReviewerInfo0213` entity contains information about a reviewer and its votes on a change.
///
/// `ReviewerInfo0213` has the same fields as `AccountInfo0213` and includes detailed account
/// information.
#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct ReviewerInfo0213 {
    /// The numeric ID of the account
    pub _account_id: Option<u64>,
    /// The full name of the user. Only set if detailed account information is requested. See
    /// option DETAILED_ACCOUNTS for change queries and option DETAILS for account queries.  
    /// (optional)
    pub name: Option<String>,
    /// The email address the user prefers to be contacted through. Only set if detailed account
    /// information is requested. See option DETAILED_ACCOUNTS for change queries and options
    /// DETAILS and ALL_EMAILS for account queries.  
    /// (optional)
    pub email: Option<String>,
    /// A list of the secondary email addresses of the user. Only set for account queries when the
    /// ALL_EMAILS option is set.  
    /// (optional)
    pub secondary_emails: Option<Vec<String>>,
    /// The username of the user. Only set if detailed account information is requested. See option
    /// DETAILED_ACCOUNTS for change queries and option DETAILS for account queries.  
    /// (optional)
    pub username: Option<String>,
    /// Whether the query would deliver more results if not limited. Only set on the last account
    /// that is returned.  
    /// (optional, not set if false)
    pub _more_accounts: Option<String>,
    /// The approvals of the reviewer as a map that maps the label names to the approval values
    /// (“-2”, “-1”, “0”, “+1”, “+2”)
    pub approvals: ReviewerInfoApprovals,
}

/// Helper struct for `ReviewerInfo` of `Aproval` information
#[derive(Deserialize, Serialize, Debug, Clone, Default)]
#[serde(deny_unknown_fields)]
pub struct ReviewerInfoApprovals {
    /// verified with
    #[serde(rename="Verified")]
    pub verified: Option<i64>,
    /// code review number
    #[serde(rename="Code-Review")]
    pub codereview: Option<i64>,
}

/// `ReviewerInfo` differs between Gerrit server/protocoll versions. This enum hold them together.
#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(untagged)]
pub enum ReviewerInfo {
    /// V2.09
    Gerrit0209(ReviewerInfo0209),
    /// V2.13
    Gerrit0213(ReviewerInfo0213),
}

/// The `AddReviewerResult` entity describes the result of adding a reviewer to a change.
#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct AddReviewerResult0209 {
    /// The newly added reviewers as a list of ReviewerInfo entities
    pub reviewers: Option<Vec<ReviewerInfo0209>>,
    /// Error message explaining why the reviewer could not be added.
    /// If a group was specified in the input and an error is returned, it means that none of the
    /// members were added as reviewer.
    pub error: Option<String>,
    /// Whether adding the reviewer requires confirmation
    pub confirm: Option<bool>,
}

/// The `AddReviewerResult` entity describes the result of adding a reviewer to a change.
#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct AddReviewerResult0213 {
    /// Value of the reviewer field from ReviewerInput set while adding the reviewer
    pub input: String,
    /// The newly added reviewers as a list of ReviewerInfo entities
    pub reviewers: Option<Vec<ReviewerInfo0213>>,
    /// The newly CCed accounts as a list of ReviewerInfo entities. This field will only appear if
    /// the requested state for the reviewer was CC **and** NoteDb is enabled on the server
    pub ccs: Option<Vec<ReviewerInfo>>,
    /// Error message explaining why the reviewer could not be added.
    /// If a group was specified in the input and an error is returned, it means that none of the
    /// members were added as reviewer.
    pub error: Option<String>,
    /// Whether adding the reviewer requires confirmation.
    pub confirm: Option<bool>,
}

/// `AddReviewerResult` differs between Gerrit server/protocoll versions. This enum hold them together.
#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(untagged)]
pub enum AddReviewerResult {
    /// V2.09
    Gerrit0209(AddReviewerResult0209),
    /// V2.09
    Gerrit0213(AddReviewerResult0213),
}

/// The `ReviewerInput` entity contains information for adding a reviewer to a change.
#[derive(Deserialize, Serialize, Debug, Clone, Default)]
pub struct ReviewerInput0209 {
    /// The ID of one account that should be added as reviewer or the ID of one group for which all
    /// members should be added as reviewers.
    /// If an ID identifies both an account and a group, only the account is added as reviewer to
    /// the change.
    pub reviewer: String,
    /// Whether adding the reviewer is confirmed.
    /// The Gerrit server may be configured to require a confirmation when adding a group as
    /// reviewer that has many members.
    pub confirmed: Option<bool>,
}

/// The `ReviewerInput` entity contains information for adding a reviewer to a change
#[derive(Deserialize, Serialize, Debug, Clone, Default)]
#[serde(deny_unknown_fields)]
pub struct ReviewerInput0213 {
    /// The ID of one account that should be added as reviewer or the ID of one group for which all
    /// members should be added as reviewers.
    /// If an ID identifies both an account and a group, only the account is added as reviewer to
    /// the change.
    pub reviewer: String,
    /// Add reviewer in this state. Possible reviewer states are REVIEWER and CC. If not given,
    /// defaults to REVIEWER.
    pub state: Option<String>,
    /// Whether adding the reviewer is confirmed.
    /// The Gerrit server may be configured to require a confirmation when adding a group as
    /// reviewer that has many members.
    pub confirmed: Option<bool>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(untagged)]
pub enum ReviewerInput {
    Gerrit0209(ReviewerInput0209),
    Gerrit0213(ReviewerInput0213),
}
