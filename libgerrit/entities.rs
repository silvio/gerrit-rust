
//! All entities of a gerrit instance
//!
//! The entities are documented on gerrit site on
//! <https://gerrit-documentation.storage.googleapis.com/Documentation/2.12.3/rest-api-changes.html#json-entities>.
//!
//! **NOTICE**: Only current needed entities are here reflected.

#![warn(missing_docs)]

use std::collections::HashMap;

/// The `AccountInfo` entity contains information about an account
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct AccountInfo {
    /// The numeric ID of the account
    /// V02.09
    pub _account_id: Option<u64>,
    /// The full name of the user.
    /// Only set if detailed account information is requested
    /// V02.09
    pub name: Option<String>,
    /// The email address the user prefers to be contacted through.
    /// Only set if detailed account information is requested
    /// V02.09
    pub email: Option<String>,
    /// The username of the user.
    /// Only set if detailed account information is requested
    /// V02.09
    pub username: Option<String>,
    /// A list of the secondary email addresses of the user. Only set for account queries when the
    /// ALL_EMAILS option is set.
    /// V02.13
    pub secondary_emails: Option<Vec<String>>,
    /// Whether the query would deliver more results if not limited. Only set on the last account
    /// that is returned.
    /// V02.13
    pub _more_accounts: Option<String>,
}

/// The `ActionInfo` entity describes a REST API call the client can make to manipulate a resource.
/// These are frequently implemented by plugins and may be discovered at runtime.
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ActionInfo {
    /// HTTP method to use with the action. Most actions use POST, PUT or DELETE to cause state
    /// changes.
    pub method: Option<String>,
    /// Short title to display to a user describing the action. In the Gerrit web interface the
    /// label is used as the text on the button presented in the UI.
    pub label: Option<String>,
    /// Longer text to display describing the action. In a web UI this should be the title
    /// attribute of the element, displaying when the user hovers the mouse.
    pub title: Option<String>,
    /// If true the action is permitted at this time and the caller is likely allowed to execute
    /// it. This may change if state is updated at the server or permissions are modified. Not
    /// present if false.
    pub enabled: Option<String>,
}

/// `ChangeInfo` helper variant to present a status of a change
#[allow(non_camel_case_types)]
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub enum ChangeInfoChangeStatus {
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
    pub optional: Option<bool>,
}

/// The `ChangeMessageInfo` entity contains information about a message attached to a change.
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ChangeMessageInfo {
    /// The ID of the message.
    /// V02.09
    pub id: String,
    /// Author of the message as an AccountInfo entity.
    /// Unset if written by the Gerrit system.
    /// V02.09
    pub author: Option<AccountInfo>,
    /// The timestamp this message was posted.
    /// V02.09
    pub date: String,
    /// The text left by the user.
    /// V02.09
    pub message: String,
    /// Value of the tag field from ReviewInput set while posting the review. NOTE: To apply
    /// different tags on on different votes/comments multiple invocations of the REST call are
    /// required.
    /// V02.13
    pub tag: Option<String>,
    /// Which patchset (if any) generated this message.
    /// V02.09
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
    pub commands: Option<HashMap<String, String>>,
}

impl FetchInfo {
    /// simplify `FetchInfo::reference`
    ///
    /// Strip 'refs/changes' and the two grouping numbers in reference of FetchInfo.
    /// Eg: `refs/changes/85/225285/1` -> `225285/1`
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use libgerrit::entities::FetchInfo;
    /// let fi = FetchInfo {
    ///     url: "http://localhost/blah".into(),
    ///     reference: "refs/changes/85/225285/1".into(),
    ///     commands: None,
    /// };
    /// assert_eq!("225285/1", fi.get_reference_string());
    /// ```
    pub fn get_reference_string(&self) -> &str {
        &self.reference.trim_left_matches("refs/changes/")[3..]
    }
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
    pub tz: i32,
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
    pub commit: Option<String>,
    /// The parent commits of this commit as a list of CommitInfo entities. In each parent only the
    /// commit and subject fields are populated.
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
    pub web_links: Option<String>,
}

/// The `FileInfo` entity contains information about a file in a patch set.
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct FileInfo {
    /// The status of the file ("A"=Added, "D"=Deleted, "R"=Renamed, "C"=Copied, "W"=Rewritten).
    /// Not set if the file was Modified ("M").
    /// V02.09
    pub status: Option<String>,
    /// Whether the file is binary.
    /// V02.09
    pub binary: Option<bool>,
    /// The old file path.
    /// Only set if the file was renamed or copied.
    /// V02.09
    pub old_path: Option<String>,
    /// Number of inserted lines.
    /// Not set for binary files or if no lines were inserted.
    /// V02.09
    pub lines_inserted: Option<u64>,
    /// Number of deleted lines.
    /// Not set for binary files or if no lines were deleted.
    /// V02.09
    pub lines_deleted: Option<String>,
    /// Number of bytes by which the file size increased/decreased.
    /// V02.13
    pub size_delta: Option<u64>,
    /// File size in bytes.
    /// V02.13
    pub size: Option<u64>,
}

/// V02.13
#[allow(non_camel_case_types)]
#[allow(missing_docs)]
#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum RevisionInfoChangeKind {
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

/// The `RevisionInfo` entity contains information about a patch set.
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct RevisionInfo {
    /// Whether the patch set is a draft.
    /// V02.09
    pub draft: Option<bool>,
    /// The change kind. Valid values are REWORK, TRIVIAL_REBASE, MERGE_FIRST_PARENT_UPDATE,
    /// NO_CODE_CHANGE, and NO_CHANGE.
    /// V02.13
    pub kind: Option<RevisionInfoChangeKind>,
    /// Whether the patch set has one or more draft comments by the calling user. Only set if draft
    /// comments is requested.
    /// V02.09
    pub has_draft_comments: Option<bool>,
    /// The patch set number.
    /// V02.09
    pub _number: u64,
    /// The timestamp of when the patch set was created.
    /// V02.13
    pub created: Option<String>,
    /// Information about how to fetch this patch set. The fetch information is provided as a map
    /// that maps the protocol name ("git", "http", "ssh") to FetchInfo entities.
    /// V02.09
    pub fetch: HashMap<String, FetchInfo>,
    /// The uploader of the patch set as an AccountInfo entity.
    /// V02.13
    pub uploader: Option<AccountInfo>,
    /// The Git reference for the patch set.
    /// V02.13
    #[serde(rename="ref")] // "ref" is a keyword
    pub reference: Option<String>,
    /// The commit of the patch set as `CommitInfo` entity.
    /// V02.09
    pub commit: Option<CommitInfo>,
    /// The files of the patch set as a map that maps the file names to `FileInfo` entities.
    /// V02.09
    pub files: Option<HashMap<String, FileInfo>>,
    /// Actions the caller might be able to perform on this revision. The information is a map of
    /// view name to ActionInfo entities.
    /// V02.09
    pub actions: Option<HashMap<String, ActionInfo>>,
    /// Indicates whether the caller is authenticated and has commented on the current revision.
    /// Only set if REVIEWED option is requested.
    /// V02.13
    pub reviewed: Option<bool>,
    /// If the COMMIT_FOOTERS option is requested and this is the current patch set, contains the
    /// full commit message with Gerrit-specific commit footers, as if this revision were submitted
    /// using the Cherry Pick submit type.
    /// V02.13
    #[serde(rename="messageWithFooter")] // "ref" is a keyword
    pub message_with_footer: Option<String>,
    /// If the PUSH_CERTIFICATES option is requested, contains the push certificate provided by the
    /// user when uploading this patch set as a PushCertificateInfo entity. This field is always
    /// set if the option is requested; if no push certificate was provided, it is set to an empty
    /// object.
    /// V02.13
    pub push_certificate: Option<PushCertificateInfo>,
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
    pub status: Option<String>,
    /// If status is set, an additional plaintext message describing the outcome of the fix.
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
    /// The name of the project. Not set if returned in a map where the project name is used as map
    /// key.
    pub name: Option<String>,
    /// The URL encoded project name.
    pub id: String,
    /// The name of the parent project.
    /// ?-<n> if the parent project is not visible (<n> is a number which is increased for each
    /// non-visible project).
    pub parent: Option<String>,
    /// The description of the project.
    pub description: Option<String>,
    /// ACTIVE, READ_ONLY or HIDDEN.
    pub state: Option<ProjectInfo_ProjectState>,
    /// Map of branch names to HEAD revisions.
    pub branches: Option<HashMap<String, String>>,
    /// Links to the project in external sites as a list of WebLinkInfo entries.
    pub web_links: Option<Vec<WebLinkInfo>>,
}

/// The `ReviewerUpdateInfo` entity contains information about updates to change’s reviewers set.
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ReviewerUpdateInfo {
    /// Timestamp of the update.
    /// V02.09
    pub updated: String,
    /// The account which modified state of the reviewer in question as AccountInfo entity.
    /// V02.09
    pub updated_by: AccountInfo,
    /// The reviewer account added or removed from the change as an AccountInfo entity.
    /// V02.09
    pub reviewer: AccountInfo,
    /// The reviewer state, one of REVIEWER, CC or REMOVED.
    /// V02.09
    pub state: ReviewerState,
}

/// The reviewers as a map that maps a reviewer state to a list of `AccountInfo` entities. Possible
/// reviewer states are REVIEWER, CC and REMOVED.
/// Only set if detailed labels are requested
#[allow(non_camel_case_types)]
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, Hash)]
pub enum ReviewerState {
    /// Users with at least one non-zero vote on the change.
    REVIEWER,
    /// Users that were added to the change, but have not voted.
    CC,
    /// Users that were previously reviewers on the change, but have been removed.
    REMOVED,
}

/// The `ChangeInfo` entity contains information about a change.
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ChangeInfo {
    /// gerritcodereview#change
    /// V02.09
    pub kind: Option<String>,
    /// The ID of the change in the format "<project>~<branch>~<Change-Id>", where project, branch
    /// and Change-Id are URL encoded. For branch the refs/heads/ prefix is omitted.
    /// V02.09
    pub id: String,
    /// The name of the project
    /// V02.09
    pub project: String,
    /// The name of the target branch.
    /// The refs/heads/ prefix is omitted.
    /// V02.09
    pub branch: String,
    /// The topic to which this change belongs.
    /// V02.09
    pub topic: Option<String>,
    /// The Change-Id of the change.
    /// V02.09
    pub change_id: String,
    /// The subject of the change (header line of the commit message).
    /// V02.09
    pub subject: String,
    /// The status of the change (NEW, SUBMITTED, MERGED, ABANDONED, DRAFT).
    /// V02.09
    pub status: ChangeInfoChangeStatus,
    /// The timestamp of when the change was created.
    /// V02.09
    pub created: String,
    /// The timestamp of when the change was last updated.
    /// V02.09
    pub updated: String,
    /// The timestamp of when the change was submitted.
    /// V02.13
    pub submitted: Option<String>,
    /// Whether the calling user has starred this change. not set if false
    /// V02.09
    pub starred: Option<bool>,
    /// A list of star labels that are applied by the calling user to this change. The labels are
    /// lexicographically sorted.
    /// V02.13
    pub stars: Option<Vec<String>>,
    /// Whether the change was reviewed by the calling user. Only set if reviewed is requested. not
    /// set if false
    /// V02.09
    pub reviewed: Option<bool>,
    /// The submit type of the change.
    /// Not set for merged changes.
    /// V02.13
    pub submit_type: Option<String>,
    /// Whether the change is mergeable.
    /// Not set for merged changes.
    /// V02.09
    pub mergeable: Option<bool>,
    /// Number of inserted lines.
    /// V02.09
    pub insertions: u32,
    /// Number of deleted lines.
    /// V02.09
    pub deletions: u32,
    /// The sortkey of the change.
    /// V02.09, not in V02.13
    pub _sortkey: Option<String>,
    /// The legacy numeric ID of the change.
    /// V02.09
    pub _number: u64,
    /// The owner of the change as an AccountInfo entity.
    /// V02.09
    pub owner: AccountInfo,
    /// Actions the caller might be able to perform on this revision. The information is a map of
    /// view name to ActionInfo entities.
    /// V02.09
    pub action: Option<ActionInfo>,
    /// Actions the caller might be able to perform on this revision. The information is a map of
    /// view name to ActionInfo entities.
    /// V02.13
    pub actions: Option<Vec<ActionInfo>>,
    /// The labels of the change as a map that maps the label names to LabelInfo entries.
    /// Only set if labels or detailed labels are requested.
    /// V02.09
    pub labels: Option<LabelInfo>,
    /// A map of the permitted labels that maps a label name to the list of values that are allowed
    /// for that label.
    /// Only set if detailed labels are requested.
    /// V02.09
    pub permitted_labels: Option<HashMap<String, Vec<String>>>,
    /// The reviewers that can be removed by the calling user as a list of AccountInfo entities.
    /// Only set if detailed labels are requested.
    /// V02.09
    pub removable_reviewers: Option<Vec<AccountInfo>>,
    /// The reviewers as a map that maps a reviewer state to a list of AccountInfo entities.
    /// Possible reviewer states are REVIEWER, CC and REMOVED.
    /// REVIEWER: Users with at least one non-zero vote on the change.
    /// CC: Users that were added to the change, but have not voted.
    /// REMOVED: Users that were previously reviewers on the change, but have been removed.
    /// Only set if detailed labels are requested.
    /// V02.13
    pub reviewers: Option<HashMap<ReviewerState, AccountInfo>>,
    /// Updates to reviewers set for the change as ReviewerUpdateInfo entities. Only set if
    /// reviewer updates are requested and if NoteDb is enabled.
    /// V02.13
    pub reviewer_updates: Option<Vec<ReviewerUpdateInfo>>,
    /// Messages associated with the change as a list of ChangeMessageInfo entities.
    /// Only set if messages are requested.
    /// V02.09
    pub messages: Option<Vec<ChangeMessageInfo>>,
    /// The commit ID of the current patch set of this change.
    /// Only set if the current revision is requested or if all revisions are requested.
    /// V02.09
    pub current_revision: Option<String>,
    /// All patch sets of this change as a map that maps the commit ID of the patch set to a
    /// RevisionInfo entity.
    /// Only set if the current revision is requested (in which case it will only contain a key for
    /// the current revision) or if all revisions are requested.
    /// V02.09
    pub revisions: Option<HashMap<String, RevisionInfo>>,
    /// Whether the query would deliver more results if not limited.
    /// Only set on either the last or the first change that is returned.
    /// V02.09
    pub _more_changes: Option<bool>,
    /// A list of ProblemInfo entities describing potential problems with this change. Only set if
    /// CHECK is set.
    /// V02.13
    pub problems: Option<Vec<ProblemInfo>>,

    // this fields are undocumented but returned from gerrit server
    // * https://bugs.chromium.org/p/gerrit/issues/detail?id=4629
    // * https://gerrit-review.googlesource.com/#/c/92152/2/Documentation/rest-api-changes.txt
    /// Not documented
    /// V02.13
    pub hashtags: Option<Vec<String>>,
    /// Not documented
    /// V02.13
    pub submittable: Option<bool>,
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
    pub topic: Option<String>,
    /// The status of the change (only NEW and DRAFT accepted here).
    // TODO: Only NEW and DRAFT allowed
    pub status: Option<String>,
    /// A {change-id} that identifies the base change for a create change operation.
    pub base_change: Option<String>,
    /// Allow creating a new branch when set to true.
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
    // TODO: only recursive, resolve, simple-two-way-in-core, ours or theirs allowed
    pub strategy: Option<String>,
}

/// The `ReviewerInfo` entity contains information about a reviewer and its votes on a change.
///
/// `ReviewerInfo` has the same fields as `AccountInfo` and includes detailed account information.
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ReviewerInfo {
    /// The numeric ID of the account
    /// V02.09
    pub _account_id: Option<u64>,
    /// The full name of the user.
    /// Only set if detailed account information is requested
    /// V02.09
    pub name: Option<String>,
    /// The email address the user prefers to be contacted through.
    /// Only set if detailed account information is requested
    /// V02.09
    pub email: Option<String>,
    /// A list of the secondary email addresses of the user. Only set for account queries when the
    /// ALL_EMAILS option is set.
    /// V02.13
    pub secondary_emails: Option<Vec<String>>,
    /// The username of the user.
    /// Only set if detailed account information is requested
    /// V02.09
    pub username: Option<String>,
    /// Whether the query would deliver more results if not limited. Only set on the last account
    /// that is returned.
    /// V02.13
    pub _more_accounts: Option<String>,
    /// gerritcodereview#reviewer
    /// V02.09
    kind: Option<String>,
    /// The approvals of the reviewer as a map that maps the label names to the approval values
    /// ("-2", "-1", "0", "+1", "+2")
    /// V02.09
    pub approvals: HashMap<String, String>,
}

/// The `AddReviewerResult` entity describes the result of adding a reviewer to a change.
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct AddReviewerResult {
    /// Value of the reviewer field from ReviewerInput set while adding the reviewer
    /// V02.13
    pub input: Option<String>,
    /// The newly added reviewers as a list of ReviewerInfo entities
    /// V02.09
    pub reviewers: Option<Vec<ReviewerInfo>>,
    /// The newly CCed accounts as a list of ReviewerInfo entities. This field will only appear if
    /// the requested state for the reviewer was CC **and** NoteDb is enabled on the server
    /// V02.13
    pub ccs: Option<Vec<ReviewerInfo>>,
    /// Error message explaining why the reviewer could not be added.
    /// If a group was specified in the input and an error is returned, it means that none of the
    /// members were added as reviewer.
    /// V02.09
    pub error: Option<String>,
    /// Whether adding the reviewer requires confirmation
    /// V02.09
    pub confirm: Option<bool>,
}

/// The `ReviewerInput` entity contains information for adding a reviewer to a change.
#[derive(Deserialize, Serialize, Debug, Clone, Default)]
pub struct ReviewerInput {
    /// The ID of one account that should be added as reviewer or the ID of one group for which all
    /// members should be added as reviewers.
    /// If an ID identifies both an account and a group, only the account is added as reviewer to
    /// the change.
    /// V02.09
    pub reviewer: String,
    /// Add reviewer in this state. Possible reviewer states are REVIEWER and CC. If not given,
    /// defaults to REVIEWER.
    /// V02.13
    pub state: Option<ReviewerState>,
    /// Whether adding the reviewer is confirmed.
    /// The Gerrit server may be configured to require a confirmation when adding a group as
    /// reviewer that has many members.
    /// V02.09
    pub confirmed: Option<bool>,
}

/// Abandon notifications to ...
/// V02.13
#[allow(non_camel_case_types)]
#[derive(Deserialize, Serialize, Debug)]
pub enum AbandonInputNotify {
    /// Noone
    NONE,
    /// only owner
    OWNER,
    /// owner and reviewer
    OWNER_REVIEWERS,
    /// to all
    ALL,
}

/// The `AbandonInput` entity contains information for abandoning a change
#[derive(Deserialize, Serialize, Debug)]
pub struct AbandonInput {
    /// Message to be added as review comment to the change when abandoning the change.
    /// V02.09
    pub message: Option<String>,
    /// Notify handling that defines to whom email notifications should be sent after the change is
    /// abandoned.
    /// Allowed values are NONE, OWNER, OWNER_REVIEWERS and ALL.
    /// If not set, the default is ALL.
    /// V02.13
    pub notify: Option<AbandonInputNotify>,
}

/// The `RestoreInput` entity contains information for restoring a change.
#[derive(Deserialize, Serialize, Debug)]
pub struct RestoreInput {
    /// Message to be added as review comment to the change when restoring the change.
    pub message: Option<String>,
}

/// The `CommentRange` entity describes the range of an inline comment
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct CommentRange {
    /// The start line number of the range
    pub start_line: u64,
    /// The character position in the start line.
    pub start_character: u64,
    /// The end line number of the range
    pub end_line: u64,
    /// The character position in the end line
    pub end_character: u64,
}

/// The `CommentInput` entity contains information for creating an inline comment
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct CommentInput {
    /// Must be gerritcodereview#comment if provided.
    /// V02.09
    pub kind: Option<String>,
    /// The URL encoded UUID of the comment if an existing draft comment should be updated.
    /// V02.09
    pub id: Option<String>,
    /// The path of the file for which the inline comment should be added.
    /// Doesn’t need to be set if contained in a map where the key is the file path.
    /// V02.09
    pub path: Option<String>,
    /// The side on which the comment should be added.
    /// Allowed values are REVISION and PARENT.
    /// If not set, the default is REVISION.
    /// V02.09
    pub side: Option<String>,
    /// The number of the line for which the comment should be added.
    /// 0 if it is a file comment.
    /// If neither line nor range is set, a file comment is added.
    /// If range is set, this should equal the end line of the range.
    /// V02.09
    pub line: Option<u64>,
    /// The range of the comment as a CommentRange entity.
    /// V02.09
    pub range: Option<CommentRange>,
    /// The URL encoded UUID of the comment to which this comment is a reply.
    /// V02.09
    pub in_reply_to: Option<String>,
    /// The timestamp of this comment.
    /// Accepted but ignored.
    /// V02.09
    pub updated: Option<String>,
    /// The comment message.
    /// If not set and an existing draft comment is updated, the existing draft comment is deleted.
    /// V02.09
    pub message: Option<String>,
    /// Value of the tag field. Only allowed on draft comment
    /// inputs; for published comments, use the tag field in
    /// link#review-input[ReviewInput]
    /// V02.13
    pub tag: Option<String>,
}

/// The `ReviewInput` entity contains information for adding a review to a revision
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ReviewInput {
    /// The message to be added as review comment.
    /// V02.09
    pub message: Option<String>,
    /// The votes that should be added to the revision as a map that maps the label names to the
    /// voting values.
    /// V02.09
    pub labels: Option<HashMap<String, i8>>,
    /// The comments that should be added as a map that maps a file path to a list of CommentInput
    /// entities.
    /// V02.09
    pub comments: Option<HashMap<String, CommentInput>>,
    /// Whether all labels are required to be within the user’s permitted ranges based on access
    /// controls.
    /// If true, attempting to use a label not granted to the user will fail the entire modify
    /// operation early.
    /// If false, the operation will execute anyway, but the proposed labels will be modified to be
    /// the "best" value allowed by the access controls.
    /// V02.09
    pub strict_labels: Option<bool>,
    /// Draft handling that defines how draft comments are handled that are already in the database
    /// but that were not also described in this input.
    /// Allowed values are DELETE, PUBLISH and KEEP.
    /// If not set, the default is DELETE.
    /// V02.09
    pub drafts: Option<String>,
    /// Notify handling that defines to whom email notifications should be sent after the review is
    /// stored.
    /// Allowed values are NONE, OWNER, OWNER_REVIEWERS and ALL.
    /// If not set, the default is ALL.
    /// V02.09
    pub notify: Option<String>,
    /// {account-id} the review should be posted on behalf of. To use this option the caller must
    /// have been granted labelAs-NAME permission for all keys of labels.
    /// V02.09
    pub on_behalf_of: Option<String>,
    /// Apply this tag to the review comment message, votes, and inline comments. Tags may be used
    /// by CI or other automated systems to distinguish them from human reviews. Comments with
    /// specific tag values can be filtered out in the web UI.
    /// V02.13
    pub tag: Option<String>,
    /// If true, comments with the same content at the same place will be omitted
    /// V02.13
    pub omit_duplicate_comments: Option<bool>,
}

/// The `ReviewInfo` entity contains information about a review
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ReviewInfo {
    /// The labels of the review as a map that maps the label names to the voting values.
    pub labels: HashMap<String, i8>,
}
