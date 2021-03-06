
use clap::{self, SubCommand, App, Arg};
use git2;
use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::BufReader;
use std::io::{self, Write};
use std::iter;
use std::ops::Deref;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::thread;
use libgerrit::error::GGRError;
use libgerrit::error::GGRResult;
use libgerrit::error::GerritError;
use libgerrit::gerrit::Gerrit;
use libgerrit::entities;
use netrc;
use url;
use config;

pub fn menu<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("topic")
    .about("topic management")
    .subcommand(SubCommand::with_name("forget")
                .about("Delete topic branch")
                .arg(Arg::with_name("branchname")
                     .help("branch name to delete")
                     .required(true)
                     .index(1)
                )
                .arg(Arg::with_name("recursive")
                     .help("recursive remove of branch")
                     .short("R")
                )
    )
    .subcommand(SubCommand::with_name("fetch")
                .about("Fetch a topic on current and all sub repositories")
                .arg(Arg::with_name("topicname")
                     .help("topic to pull")
                     .required(true)
                     .takes_value(true)
                 )
                .arg(Arg::with_name("branchname")
                     .help("local branch name, without this the remote branch name is taken")
                     .short("b")
                     .long("branch")
                     .takes_value(true)
                )
                .arg(Arg::with_name("force")
                     .help("local existing branches are force moved")
                     .short("f")
                     .long("force")
                )
                .arg(Arg::with_name("track")
                     .help("Set tracking branch for newly created local branches (or all branches with --force)")
                     .long("track")
                     .takes_value(true)
                 )
                .arg(Arg::with_name("closed")
                    .help("Search changes within closed reviews")
                    .long("closed")
                )
                .arg(Arg::with_name("all")
                     .help("pull all versions of all changes in a topic as tags")
                     .long("all")
                     .short("a")
                 )
    )
    .subcommand(SubCommand::with_name("history")
               .about("Fetch all versions of all changes of a topic to tags")
               .arg(Arg::with_name("topicname")
                    .help("topic to pull")
                    .required(true)
                    .index(1)
               )
               .after_help("* the tags are added in format ggr/<topicname>/<changeid>/<version> \n\
                            * remove all tags of a topic via `git submodule foreach 'git tag -l 'ggr/<topicname>' | xargs git tag -d`")
    )
    .subcommand(SubCommand::with_name("checkout")
                .about("Checkout a branch on current and all sub repositories")
                .arg(Arg::with_name("branchname")
                     .help("local branch to checkout")
                     .required(true)
                     .takes_value(true)
                )
    )
    .subcommand(SubCommand::with_name("reviewer")
                .about("manage reviewer of a topic")
                .arg(Arg::with_name("topicname")
                     .help("topic name for reviewer manipulation")
                     .required(true)
                     .takes_value(true)
                     .index(1)
                 )
                .arg(Arg::with_name("reviewers")
                     .help("List of reviewers, comma separated. '~' in front of mailaddress remove them like '~admin@example.com'")
                     .long("reviewer")
                     .short("r")
                     .alias("reviewers")
                     .takes_value(true)
                     .multiple(true)
                )
                .arg(Arg::with_name("verbose")
                     .help("all reviewers with verify information")
                     .long("verbose")
                     .short("v")
                )
    )
    .subcommand(SubCommand::with_name("abandon")
                .about("Abandon a topic")
                .arg(Arg::with_name("topicname")
                     .help("topic name to abandon/restore")
                     .required(true)
                     .takes_value(true)
                     .index(1)
                )
                .arg(Arg::with_name("message")
                     .help("message for abandon/restore operation")
                     .short("m")
                     .long("message")
                     .takes_value(true)
                )
    )
    .subcommand(SubCommand::with_name("restore")
                .about("Restore a topic")
                .arg(Arg::with_name("topicname")
                     .help("topic name to abandon/restore")
                     .required(true)
                     .takes_value(true)
                     .index(1)
                )
                .arg(Arg::with_name("message")
                     .help("message for abandon/restore operation")
                     .short("m")
                     .long("message")
                     .takes_value(true)
                )
    )
    .subcommand(SubCommand::with_name("verify")
                .about("verify topic")
                .arg(Arg::with_name("topicname")
                     .help("topicname for verify of a complete topic")
                     .required(true)
                     .takes_value(true)
                     .index(1)
                )
                .arg(Arg::with_name("code-review")
                     .help("change 'Code-Review' label")
                     .takes_value(true)
                     .short("c")
                     .long("code-review")
                     .possible_values(&["~2", "~1", "0", "1", "+1", "2", "+2"])
                )
                .arg(Arg::with_name("label")
                     .help("add other label and value: 'Verify: +1'")
                     .takes_value(true)
                     .short("l")
                     .long("label")
                )
                .arg(Arg::with_name("message")
                     .help("message append to all changes")
                     .takes_value(true)
                     .short("m")
                     .long("message")
                )
    )
}

/// manage subfunction of `topic` command
///
/// Currently implemented sub commands:
///
/// * create
/// * forget
/// * pull
pub fn manage(x: &clap::ArgMatches, config: &config::Config) -> GGRResult<()> {
    match x.subcommand() {
        ("forget", Some(y)) => { forget(y) },
        ("fetch", Some(y)) => { fetch(y, config) },
        ("history", Some(y)) => { history(y, config) },
        ("checkout", Some(y)) => { checkout(y, config) },
        ("reviewer", Some(y)) => { reviewer(y, config) },
        ("abandon", Some(y)) => { abandon(y, config) },
        ("restore", Some(y)) => { restore(y, config) },
        ("verify", Some(y)) => { verify(y, config) },
        _ => {
            println!("{}", x.usage());
            Ok(())
        },
    }
}

/// delete topics
fn forget(y: &clap::ArgMatches) -> GGRResult<()> {
    let branchname = match y.value_of("branchname") {
        Some(x) => x,
        None => return Err(GGRError::General("Bad branchname".into())),
    };

    let repo = git2::Repository::discover(".")?;
    let recursive = y.is_present("recursive");

    /* remove branch on the current repository */
    print!("* delete {} in {:?}: ", branchname, repo.path());
    match forget_branch(&repo, branchname) {
        Ok(_) => { println!("OK"); },
        Err(x) => { println!("FAILED, ({})", x); }
    }

    /* remove branch on all submodules */
    if recursive {
        let submodules = repo.submodules()?;

        for sm in &submodules {
            let reposub = sm.open()?;

            print!("* delete {} in {}: ", branchname, reposub.path().file_name().unwrap().to_string_lossy());
            match forget_branch(&reposub, branchname) {
                Ok(_) => { println!("OK"); },
                Err(x) => { println!("FAILED, ({})", x); }
            }
        }
    }
    Ok(())
}

fn forget_branch(repo: &git2::Repository, branchname: &str) -> GGRResult<()>
{
    repo.find_branch(branchname, git2::BranchType::Local)?
        .delete()
        .map_err(|x| { GGRError::from(x) })
}

pub enum OwnedOrRef<'a, T: 'a>
{
    Ref(&'a T),
    Owned(T),
}

impl<'a, T: 'a> Deref for OwnedOrRef<'a, T>
{
    type Target = T;

    fn deref(&self) -> &T {
        match *self {
            OwnedOrRef::Ref(val) => val,
            OwnedOrRef::Owned(ref val) => val,
        }
    }
}

/// fetch topics
fn fetch(y: &clap::ArgMatches, config: &config::Config) -> GGRResult<()> {
    if !config.is_root() {
        return Err(GGRError::General("You have to run topic::fetch on the main/root repository".into()));
    }

    let topicname = y.value_of("topicname").expect("no or bad topicname");
    let force = y.is_present("force");
    let local_branch_name = y.value_of("branchname").unwrap_or(topicname);
    let tracking_branch_name = y.value_of("track");
    let closed = y.is_present("closed");
    let all = y.is_present("all");

    if all {
        let _ = history(y, config);
    }

    let mut gerrit = Gerrit::new(config.get_base_url());
    fetch_topic(&mut gerrit, topicname, local_branch_name, force, tracking_branch_name, closed)
}

/// fetch history of a topic
fn history(y: &clap::ArgMatches, config: &config::Config) -> GGRResult<()> {
    if !config.is_root() {
        return Err(GGRError::General("You have to run topic::fetch on the main/root repository".into()));
    }

    let topicname = y.value_of("topicname").expect("no or bad topicname").to_owned();
    let mut gerrit = Gerrit::new(config.get_base_url());

    let mut changes = gerrit.changes();
    let query_part = vec!(format!("topic:{}", topicname));

    let changeinfos = changes.query_changes(Some(query_part), Some(vec!("ALL_REVISIONS".into(), "ALL_COMMITS".into())))?;

    if changeinfos.is_empty() {
        println!("topic '{}' not found", topicname);
        return Ok(());
    }

    for ci in changeinfos {
        println!("* working on {} {:20} ({:?})", ci.change_id, ci.subject, ci.status);
        let current_revision = match ci.current_revision {
            Some(ref x) => x,
            None => { println!("  no current revision set. No work on this changeid"); continue; }
        };

        let mut children = vec!();

        let cirevisions = ci.revisions.unwrap();

        for (revision, revisioninfo) in cirevisions {
            let current_revision = current_revision.clone();
            let topicname = topicname.clone();
            let cistatus = ci.status.clone();
            let dryrun = *config.dry_run();

            children.push(thread::spawn(move || {
                let is_abandoned = cistatus == entities::ChangeInfoChangeStatus::ABANDONED;
                let is_newest = (revision == current_revision) && (!is_abandoned);
                let mark = if is_newest { ">" } else { " " };

                let mut outstr = format!("  {} {} ", mark, revision);

                for (fetchtype, fetchinfo) in &revisioninfo.fetch {
                    if fetchtype.starts_with("http") {
                        match do_fetch_from_repo(fetchinfo, &topicname, TagOrBranch::Tag, None, false, dryrun) {
                            Err(x) => {
                                outstr.push_str(&format!("FAILED: {}", x));
                            },
                            Ok(msg) => outstr.push_str(&msg),
                        };
                    }
                }
                println!("{}", outstr);
            }));
        }

        for child in children {
            let _ = child.join();
        };
    }

    Ok(())
}

#[derive(Debug, PartialEq)]
enum TagOrBranch {
    Tag,
    Branch,
}


/// fetch via fetchinfo entity a tag or branch, and for branches it can set tracking information.
fn do_fetch_from_repo(fetchinfo: &entities::FetchInfo, topic: &str, tag_or_branch: TagOrBranch, tracking_branch_name: Option<&str>, force: bool, dryrun: bool) -> GGRResult<String>
{
    debug!("history fetch {:?}", fetchinfo);

    let main_repo = git2::Repository::open(".")?;
    let repo = history_find_repo_for_fetchinfo_url(&main_repo, &fetchinfo.url)?;
    let forcemark = if force { "+" } else { "" };

    let (refspecs, name) = match tag_or_branch {
        TagOrBranch::Tag => {
            (
                format!("{}:refs/tags/ggr/{}/{}", fetchinfo.reference, topic, fetchinfo.get_reference_string()),
                format!("ggr/{}/{}", topic, fetchinfo.get_reference_string())
            )
        },
        TagOrBranch::Branch => {
            (
                format!("{}{}:{}", forcemark, fetchinfo.reference, topic),
                String::from(topic)
            )
        },
    };

    /* we have found the rpeository. we can now fetch and tag the revision. */
    let mut cb = git2::RemoteCallbacks::new();
    cb.credentials(|url, username, allowed| {
        debug!("credential callback: {} / {:?} / {:?}", url, username, allowed);

        let homefolder = env::home_dir().ok_or(git2::Error::from_str("set HOME environment variable for searching of netrc"))?;
        let mut netrcfile = PathBuf::new();
        netrcfile.push(homefolder);
        netrcfile.push(".netrc");

        if !netrcfile.exists() {
            return Err(git2::Error::from_str(&format!("cannot find .netrc file at {:?}", netrcfile.as_path())));
        }

        debug!("found .netrc file");

        let f = File::open(netrcfile.as_path())
            .map_err(|x| { git2::Error::from_str(&format!("file: {}", x)) } )?;
        let reader = BufReader::new(f);

        let netrc = netrc::Netrc::parse(reader)
            .map_err(|x| { git2::Error::from_str(&format!("{:?}", x)) } )?;

        let repourl = url::Url::parse(url)
            .map_err(|x| { git2::Error::from_str(&format!("{}", x)) } )?;
        for (_, &(ref machinehost, ref machine)) in netrc.hosts.iter().enumerate() {
            debug!("check machinehost: {}", machinehost);
            if repourl.host_str() == Some(machinehost) {
                let password = machine.password.as_ref().ok_or(git2::Error::from_str(&format!("no password for machine {} in netrc", machinehost)))?;
                let passwordplace = iter::repeat("*").take(password.len()).collect::<String>();
                debug!("use credentials for login: '{}', with password (hidden): '{}'", &machine.login, passwordplace);
                return git2::Cred::userpass_plaintext(&machine.login, password);
            }
        }

        Err(git2::Error::from_str(&format!("no correct netrc entry for repository {} found.", url)))
    });

    let mut fetchoptions = git2::FetchOptions::new();
    fetchoptions.prune(git2::FetchPrune::Off)
        .update_fetchhead(false)
        .download_tags(git2::AutotagOption::None)
        .remote_callbacks(cb);

    // TODO: check tag exists

    let workdir = repo.workdir().ok_or(format!("no workdir for '{}' found", repo.path().to_string_lossy()))?
        .file_name().unwrap();
    if !dryrun {
        match repo.find_remote("origin")?.fetch(&[&refspecs], Some(&mut fetchoptions), Some("")) {
            Ok(_) => {
                if tag_or_branch == TagOrBranch::Branch {
                    if let Ok(mut branch) = repo.find_branch(&name, git2::BranchType::Local) {
                        let _ = branch.set_upstream(tracking_branch_name);
                    };
                };
                Ok(format!("OK, pulled '{}' as {:?} '{}' into {}", fetchinfo.reference, tag_or_branch, name, workdir.to_string_lossy()))
            },
            Err(x) => {
                Err(GGRError::General(format!("FAILED! Could not pull {} at {} ({})", refspecs, fetchinfo.url, x.message())))
            },
        }
    } else {
        Ok(format!("OK, (dry-run) pulled '{}' as {:?} '{}' into {}", fetchinfo.reference, tag_or_branch, name, workdir.to_string_lossy()))
    }
}

fn history_extract_projectname<P>(path: &P) -> Option<&str>
where P: AsRef<Path>
{
    path.as_ref()
        .file_name()
        .and_then(|p| p.to_str())
}

/// find a repository in a repo/submodule structure with pointer to url
fn history_find_repo_for_fetchinfo_url<'a>(main_repo: &'a git2::Repository, url: &str) -> GGRResult<OwnedOrRef<'a, git2::Repository>>
{
    debug!("find repo for url {}, starts with {:?}", url, main_repo.path());

    let mut found;
    let url = history_extract_projectname(&url).ok_or(format!("problem to extract projectname form url {}", url))?;
    let urls = vec!(
        String::from(url),
        format!("{}.git", url),
    );

    debug!("transformed urls: '{:?}'", urls);

    // check mainrepo handles url
    let main_repo_remotesnames = main_repo.remotes()?;
    for main_repo_remotename in main_repo_remotesnames.iter() {
        if main_repo_remotename.is_none() { continue };
        let main_repo_remote = main_repo.find_remote(main_repo_remotename.unwrap())?;
        found = match main_repo_remote.url() {
            Some(main_repo_url) => {
                debug!("url:{} === main-url:{}", url, main_repo_url);
                urls.contains(&String::from(history_extract_projectname(&main_repo_url).ok_or(format!("problem to extract projectname form url {}", main_repo_url))?))
            },
            None => false,
        };
        if found {
            debug!("found: {:?}", main_repo.path());
            return Ok(OwnedOrRef::Ref(main_repo));
        };
    }

    // check all submodules for main_repo
    let submodules = main_repo.submodules()?;

    for submodule in submodules {
        found = match submodule.url() {
            Some(submodule_url) => {
                debug!("url:{} === subm-url:{}", url, submodule_url);
                urls.contains(&String::from(history_extract_projectname(&submodule_url).ok_or(format!("problem to extract projectname form url {}", submodule_url))?))
            },
            None => false,
        };
        if found {
            debug!("found: {:?}", submodule.path());
            match submodule.open() {
                Ok(x) => {
                    return Ok(OwnedOrRef::Owned(x))
                },
                Err(x) => {
                    return Err(String::from(x.message()).into())
                },
            }
        }
    }

    debug!("not found");
    Err("The url doesn't match main repo and submodules of mainrepo".into())
}

/// checkout topics
fn checkout(y: &clap::ArgMatches, config: &config::Config) -> GGRResult<()> {
    if !config.is_root() {
        return Err(GGRError::General("You have to run topic::checkout on the main/root repository".into()));
    }

    let branchname = y.value_of("branchname").unwrap();
    checkout_topic(branchname)
}

/// show and manipulate reviewer
fn reviewer(y: &clap::ArgMatches, config: &config::Config) -> GGRResult<()> {
    let topicname = y.value_of("topicname").expect("you need a topicname");
    let verbose = y.is_present("verbose");

    let mut gerrit = Gerrit::new(config.get_base_url());
    if let Ok(cis) = gerrit.changes().query_changes(Some(vec!(&format!("topic:{}", topicname)[..])), None) {

        let mut children = Vec::new();

        // manipulate reviewer for topic
        if let Some(ref reviewerlist) = y.values_of_lossy("reviewers") {
            for ci in cis {
                let reviewerlist = reviewerlist.clone();
                let mut gerrit = gerrit.clone();
                children.push(thread::spawn(move || {
                    for reviewer in reviewerlist {
                        let remove = reviewer.starts_with('~');

                        if remove {
                            let reviewer = &reviewer[1..];
                            if let Err(res) = gerrit.changes().delete_reviewer(&ci.change_id, reviewer) {
                                /*
                                 * delete_changes returnes a empty body and a status code. A empty body
                                 * cannot deserialized its break with a error message
                                 * "JsonError(ErrorImpl { code: EofWhileParsingValue, line: 1, column: 0 })"
                                 *
                                 * Now we destructure the objects and check for status code 204 (no
                                 * content) and overwrite this to be okay.
                                 */
                                match res {
                                    GGRError::GerritApiError(ref x) => {
                                        match *x {
                                            GerritError::GerritApi(ref status, ref text) => {
                                                if *status >= 400 {
                                                    println!("{}, ({}: {})", reviewer, status, text);
                                                } else {
                                                    println!("* {:5.5} [{:20.20}] reviewer '{}' removed", ci.change_id, ci.subject, reviewer);
                                                }
                                            },
                                            ref err => {
                                                println!("Other error: {:?}", err);
                                            },
                                        }
                                    },
                                    x => { println!("Other error: {:?}", x);}
                                };

                            } else {
                                println!("* {:5.5} [{:20.20}] reviewer '{}' removed", ci.change_id, ci.subject, reviewer);
                            };
                        } else {
                            match gerrit.changes().add_reviewer(&ci.change_id, &reviewer) {
                                Ok(addreviewerresult) => {

                                    match addreviewerresult.reviewers {
                                        Some(reviewerret) => {
                                            for r in reviewerret {
                                                println!("* {:5.5} [{:20.20}] reviewer {}, {}, {}: added",
                                                         ci.change_id,
                                                         ci.subject,
                                                         r.name.unwrap_or_else(|| "unkown name".into()),
                                                         r.email.unwrap_or_else(|| "unkown mail".into()),
                                                         r._account_id.unwrap_or(99999999));
                                            }
                                        },
                                        None => {
                                            println!("* {:5.5} [{:20.20}] reviewer '{}' not added: {}",
                                                     ci.change_id,
                                                     ci.subject,
                                                     reviewer,
                                                     addreviewerresult.error.unwrap_or_else(|| "No error message from gerrit server provided".into()));
                                        },
                                    };
                                },
                                Err(e) => {
                                    println!("Problem to add '{}' as reviewer: {}", reviewer, e);
                                },
                            }
                        }
                    }
                }));
            }

            for child in children {
                let _ = child.join();
            }

            return Ok(());
        }

        // only list reviewers
        debug!("threads: {}", cis.len());
        for ci in cis {
            let mut gerrit = gerrit.clone();
            children.push(thread::spawn(move || {
                let mut out = format!("reviewer for '{}':\n", ci.subject);
                if let Ok(reviewers) = gerrit.changes().get_reviewers(&ci.id) {
                    let mut reviewer_list = Vec::new();
                    for reviewer in reviewers {
                        let (name, email, approval) = (
                            reviewer.name.unwrap_or_else(|| "unknown".into()),
                            reviewer.email.unwrap_or_else(|| "unknown".into()),
                            reviewer.approvals
                        );
                        reviewer_list.push(name.clone());

                        if verbose {
                            for (approvei_label, approve_value) in &approval {
                                out.push_str(&format!("  * {:20.20} {:20.20} {:>5.5}\n", email, approvei_label.trim(), approve_value.trim()));
                            }
                        }
                    }
                    if ! verbose {
                        out.push_str("  ".into());
                        for reviewer in reviewer_list {
                            out.push_str(&format!("{}, ", reviewer));
                        }
                        out.push_str("\n".into());
                    }
                    println!("{}", out);
                }
            }));
        }

        for child in children {
            let _ = child.join();
        }
    } else {
        println!("no changes for '{}' found", topicname);
    }

    Ok(())
}

/// abandon a topic
fn abandon(y: &clap::ArgMatches, config: &config::Config) -> GGRResult<()> {
    let topicname = y.value_of("topicname").expect("problem with topic name");
    let message = y.value_of("message");

    debug!("abandon topic:{}, message:{}", topicname, message.unwrap_or(""));

    let mut gerrit = Gerrit::new(config.get_base_url());

    if let Ok(cis) = gerrit.changes().query_changes(Some(vec!(&format!("topic:{}", topicname)[..])), None) {
        for ci in cis {

            let (abid, absubject, abcause) = match gerrit.changes().abandon_change(&ci.change_id, message, None) {
                Ok(ciret) => (ciret.change_id, ciret.subject, None),
                Err(x) => (ci.change_id, ci.subject, Some(x)),
            };

            match abcause {
                None => {
                    println!("* {:5.5} [{:20.20}] abandoned", abid, absubject);
                },
                Some(x) => {
                    println!("* {:5.5} [{:20.20}] not abandoned: {}", abid, absubject, x);
                },
            };
        }
    }

    Ok(())
}

/// restore a topic
fn restore(y: &clap::ArgMatches, config: &config::Config) -> GGRResult<()> {
    let topicname = y.value_of("topicname").expect("problem with topic name");
    let message = y.value_of("message");

    debug!("restore topic:{}, message:{}", topicname, message.unwrap_or(""));

    let mut gerrit = Gerrit::new(config.get_base_url());

    if let Ok(cis) = gerrit.changes().query_changes(Some(vec!(&format!("topic:{}", topicname)[..])), None) {
        for ci in cis {

            let (abid, absubject, abcause) = match gerrit.changes().restore_change(&ci.change_id, message) {
                Ok(ciret) => (ciret.change_id, ciret.subject, None),
                Err(x) => (ci.change_id, ci.subject, Some(x)),
            };

            match abcause {
                None => {
                    println!("* {:5.5} [{:20.20}] restored", abid, absubject);
                },
                Some(x) => {
                    println!("* {:5.5} [{:20.20}] not restored: {}", abid, absubject, x);
                },
            };
        }
    }

    Ok(())
}

/// verify a topic
fn verify(y: &clap::ArgMatches, config: &config::Config) -> GGRResult<()> {
    let topicname = y.value_of("topicname").expect("problem with topicname");
    let message = y.value_of("message");

    let review = {
        let mut r = entities::ReviewInfo {
            labels: HashMap::new(),
        };

        if let Some(label) = y.value_of("label") {
            if label.contains(':') {
                let mut labelvalue = label.splitn(2, ':');
                let label = labelvalue.next().unwrap_or("").trim();
                let value = labelvalue.next().unwrap_or("").trim();

                if ! label.is_empty() && ! value.is_empty() {
                    let value = match value {
                        "~2" | "-2" => -2,
                        "~1" | "-1" => -1,
                        "0" | "+0" | "-0" => 0,
                        "1" | "+1" => 1,
                        "2" | "+2" => 2,
                        _ => return Err(GGRError::General("Wrong value for label. Only support for (-2,-1,0,1,2)".into())),
                    };
                    r.labels.entry(label.into()).or_insert(value);
                }
            }
        };

        if let Some(codereview) = y.value_of("code-review") {
            let codereview = match codereview {
                "~2" => -2,
                "~1" => -1,
                "0" => 0,
                "1" => 1,
                "2" => 2,
                _ => return Err(GGRError::General("Wrong code-review parameter".into())),
            };
            r.labels.entry("Code-Review".into()).or_insert(codereview);
        };

        if r.labels.is_empty() {
            None
        } else {
            Some(r)
        }
    };

    let mut gerrit = Gerrit::new(config.get_base_url());
    let mut changes = gerrit.changes();

    if let Ok(changeinfos) = changes.query_changes(Some(vec!(&format!("topic:{}", topicname)[..])), Some(vec!("CURRENT_REVISION"))) {
        /* overall review result for the commit */
        let mut overall_review: HashMap<String /*label*/, (i8,i8) /* min/max */> = HashMap::new();

        for ci in changeinfos {
            debug!("{:?}", ci);
            let (id, changeid, revision, subject) = (
                ci.id.clone(),
                ci.change_id.clone(),
                ci.current_revision.unwrap_or_else(|| "".into()),
                ci.subject
            );

            let changes = gerrit.changes();

            if message.is_none() && review.is_none() {
                // neither review or message is set, we retrieve review information

                match changes.get_reviewers(&id) {
                    Ok(reviewerinfos) => {
                        /* a list of reviews for one changeset */
                        let mut changeinfo_review: HashMap<String /* label */, Vec<String> /* list of reviews */> = HashMap::new();

                        for ri in reviewerinfos {
                            for (label, review) in ri.approvals {
                                let review = String::from(review.trim());
                                let entry = changeinfo_review.entry(label.clone()).or_insert_with(Vec::new);
                                if let Ok(review_int) = review.parse() {
                                    entry.push(review);

                                    let overall = overall_review.entry(label.clone()).or_insert((0,0));
                                    if review_int < overall.0 {
                                        overall.0 = review_int;
                                    }
                                    if review_int > overall.1 {
                                        overall.1 = review_int;
                                    }
                                } else {
                                    debug!("This review is not convertible to int: {:?}", review);
                                }
                            };
                        };

                        println!("* {:5.5} {}:", changeid, subject);
                        for (label, review) in changeinfo_review {
                            let mut sortreview = review.clone();
                            sortreview.sort();
                            println!("  {:10.10} -> {:?}", label, sortreview);
                        };
                    },
                    Err(err) => {
                        println!("Problem to recive reviewers: {}", err);
                        return Err(err);
                    }
                };
            } else {
                // message and/or review is set we push them to the gerrit server
                match changes.set_review(&id, &revision, message, review.clone()) {
                    Ok(reviewinfo) => println!("* {:5.5} {:20.20}, applied: {:?}", changeid, subject, reviewinfo.labels),
                    Err(err) => println!("* {:5.5} {:20.20}, not applied: {}", changeid, subject, err),
                };
            }
        }

        // Isn't empty only when review and message was empty (we want to show the review results).
        if !overall_review.is_empty() {
            println!("\nOverall min/max:");

            for (label, review) in overall_review {
                println!("* {label:10.10}: {min:+}/{max:+}", label=label, min=review.0, max=review.1);
            }
        }
    }

    Ok(())
}

/// Convenient function to fetch topic `topicname` to branch `local_branch_name`.
///
/// If branch exists and `force` is true, the branch is moving to new position.
fn fetch_topic(gerrit: &mut Gerrit, topicname: &str, local_branch_name: &str, force: bool, tracking_branch_name: Option<&str>, closed: bool) -> GGRResult<()> {
    trace!("fetch_topic: topicname:{} local_branch_name:{} force:{} tracking_branch_name:{:?} closed:{}",
           topicname, local_branch_name, force, tracking_branch_name, closed);

    let mut changes = gerrit.changes();

    let mut query_part = vec!(format!("topic:{}", topicname));
    if !closed {
        query_part.push("status:open".into());
    }

    let changeinfos = changes.query_changes(Some(query_part), Some(vec!("CURRENT_REVISION".into(), "CURRENT_COMMIT".into())))?;
    if changeinfos.is_empty() {
        println!("topic '{}' not found", topicname);
        return Ok(());
    }
    fetch_changeinfos(&changeinfos, force, local_branch_name, tracking_branch_name)
}

/// Convenient function to pull one or more `changeids`
///
/// all ancestore commits are pulled from gerrit server too.
pub fn fetch_changeinfos(changeinfos: &[entities::ChangeInfo], force: bool, local_branch_name: &str, tracking_branch_name: Option<&str>) -> GGRResult<()> {
    let project_tip = project_tip(changeinfos).unwrap();

    // try to fetch topic for main_repo and all submodules
    'next_ptip: for (p_name, p_tip) in project_tip {
        print!("fetch {} for {} ... ", p_name, p_tip);
        // check for root repository
        if let Ok(main_repo) = git2::Repository::open(".") {
            // check changes on root repository
            match fetch_from_repo(&main_repo, changeinfos, force, local_branch_name, &p_name, &p_tip, tracking_branch_name) {
                Ok((true, x)) => {
                    println!("OK ({})", x);
                    continue;
                },
                Ok((false, m)) => {
                    println!("FAILED\n  Error: {}", m.trim());
                },
                Err(r) => {
                    // hide all other errors
                    let r = r.to_string();
                    if !r.is_empty() {
                        println!("FAILED\nError: {}", r.to_string().trim());
                    }
                }
            };

            // check for submodules
            if let Ok(smodules) = main_repo.submodules() {
                for smodule in smodules {
                    if let Ok(sub_repo) = smodule.open() {
                        match fetch_from_repo(&sub_repo, changeinfos, force, local_branch_name, &p_name, &p_tip, tracking_branch_name) {
                            Ok((true, _)) => {
                                println!("OK");
                                continue 'next_ptip;
                            },
                            Ok((false, m)) => {
                                println!("FAILED\n  Error: {}", m.trim());
                                continue;
                            },
                            Err(r) => {
                                let r = r.to_string();
                                if !r.is_empty() {
                                    println!("FAILED\nError: {}", r.to_string().trim());
                                }
                            }
                        }
                    } else {
                        println!("{} not opened", smodule.name().unwrap());
                    }
                }
            }
        }
        println!("repo not a submodule, unknown repo or commit");
    }

    Ok(())
}

/// convenient function to pull a `project_tip` from a `repo`, if `basename(repo.url)` same as
/// `project_name` is.
///
/// The `project_tip` is the head of a topic within a repository
///
/// returns `true` if something is pulled, and `false` if no pull was executed. The String object
/// is a status message.
fn fetch_from_repo(repo: &git2::Repository, ci: &[entities::ChangeInfo], force: bool, local_branch_name: &str, project_name: &str, project_tip: &str, tracking_branch_name: Option<&str>) -> GGRResult<(bool, String)> {
    trace!("repo-path:{:?}, project_name:{}, project_tip:{}", repo.path().file_name(), project_name, project_tip);
    if repo.is_bare() {
        return Err(GGRError::General(format!("repository path '{:?}' is bare, we need a workdir", repo.path())));
    }

    for remote_name in repo.remotes().unwrap().iter() {
        if let Ok(remote) = repo.find_remote(remote_name.unwrap()) {
            let url = remote.url().unwrap().to_owned();
            let check_project_names = vec!(
                project_name.into(),
                format!("{}.git", project_name)
            );

            if check_project_names.contains(&String::from(url_to_projectname(&url).unwrap())) {
                for entity in ci {
                    let current_revision = match entity.current_revision {
                        None => { continue },
                        Some(ref x) => x,
                    };

                    if project_tip != current_revision { continue };

                    let cirevisions = entity.clone().revisions.unwrap();
                    for (ref revision, ref revisioninfo) in cirevisions {
                        if revision != current_revision { continue };

                        for (fetchtype, fetchinfo) in &revisioninfo.fetch {
                            if fetchtype.starts_with("http") {
                                match do_fetch_from_repo(fetchinfo, local_branch_name, TagOrBranch::Branch, tracking_branch_name, force, false) {
                                    Err(x) => return Err(x),
                                    Ok(x) => {
                                        return Ok((true, x))
                                    },
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    Err(GGRError::General("".into()))
}

/// returns a `HashMap` with project and tip of a topic.changeset
fn project_tip(changes: &[entities::ChangeInfo]) -> GGRResult<HashMap<String, String>> {
    // find involved projects
    let mut list_of_projects = Vec::new();
    for element in changes {
        if !list_of_projects.contains(&element.project) {
            list_of_projects.push(element.project.clone());
        }
    }

    /* TODO: rewrite */
    // find tip of every project
    let mut project_tip: HashMap<String, String> = HashMap::new();
    for project in list_of_projects {
        // find in entities the last change of every project for this topic
        let mut list_all_parents = Vec::new();
        // fill a list with all parents
        for element in changes {
            if let Some(ref cur_revision) = element.current_revision {
                if let Some(ref revisions) = element.revisions {
                    if let Some(cur_revision) = revisions.get(cur_revision) {
                        if let Some(ref commit) = cur_revision.commit {
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

        /* TODO: rewrite */
        for element in changes {
            if element.project == *project {
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

pub fn entity_from_commit<'ci>(changes: &'ci [entities::ChangeInfo], commit: &str) -> GGRResult<&'ci entities::ChangeInfo> {
    for element in changes {
        if let Some(ref revisions) = element.revisions {
            for rev in revisions.keys() {
                if rev == commit {
                    return Ok(element);
                }
            }
        }
    }

    Err(GGRError::General("no entity found".into()))
}


/// Convenient function to checkout a topic
pub fn checkout_topic(branchname: &str) -> GGRResult<()> {
        if let Ok(main_repo) = git2::Repository::open(".") {
            let mut out_ok: Vec<String> = Vec::new();
            let mut out_ko: Vec<String> = Vec::new();

            print!("try checkout on main repo ... ");
            match checkout_repo(&main_repo, branchname) {
                Ok(_) => {
                    println!("OK");
                    if ! main_repo.submodules().ok().unwrap_or_default().is_empty() {
                        println!("git submodule update ...");
                        let output_submodule_update = Command::new("git")
                            .arg("submodule")
                            .arg("update")
                            .arg("--recursive")
                            .arg("--init")
                            .output()?;

                        if ! output_submodule_update.stdout.is_empty() {
                            println!("  submodule update stdout:\n{}", String::from_utf8_lossy(&output_submodule_update.stdout));
                        }
                        if ! output_submodule_update.stderr.is_empty() {
                            println!("  submodule update stderr:\n{}", String::from_utf8_lossy(&output_submodule_update.stderr));
                        }
                    }
                },
                Err(m) => println!("{} -> {}", main_repo.path().to_str().unwrap(), m.to_string().trim()),
            }

            if let Ok(smodules) = main_repo.submodules() {
                print!("try checkout submodules: ");
                if ! smodules.is_empty() {
                    for smodule in smodules {
                        if let Ok(sub_repo) = smodule.open() {
                            match checkout_repo(&sub_repo, branchname) {
                                Ok(_) => {
                                    print!("+");
                                    out_ok.push(format!("{:?}", smodule.name().unwrap_or("unknown repository")))
                                },
                                Err(m) => {
                                    print!("-");
                                    out_ko.push(format!("{:?} -> {}", smodule.name().unwrap_or("unknown repository"), m.to_string().trim()))
                                },
                            };
                            let _ = io::stdout().flush();
                        }
                    }
                    println!("\n");

                    if !out_ko.is_empty() {
                        println!("Not possible to checkout '{}' on this repositories:", branchname);
                        for entry in out_ko {
                            println!("* {}", entry);
                        }
                    }

                    if !out_ok.is_empty() {
                        println!("\nSuccessfull checkout of '{}' on this repositories:", branchname);
                        for entry in out_ok {
                            println!("* {}", entry);
                        }
                    } else {
                        println!("No checkout happened");
                    }
                } else {
                    println!("no submodules used");
                }
            }
        }
    Ok(())
}

/// convenient function to checkout a `branch` on a `repo`. If `print_status` is true, messages are
/// printed
fn checkout_repo(repo: &git2::Repository, branchname: &str) -> GGRResult<()> {
    if repo.is_bare() {
        return Err(GGRError::General("repository needs to be a workdir and not bare".into()));
    }

    let output_checkout = try!(Command::new("git")
        .current_dir(repo.workdir().unwrap())
        .arg("checkout")
        .arg(branchname)
        .output());

    if output_checkout.status.success() {
        return Ok(());
    }

    Err(GGRError::General(String::from_utf8_lossy(&output_checkout.stderr).into()))
}

/// returns basename of a project from a url (eg.: https://localhost/test -> test)
fn url_to_projectname(url: &str) -> Option<&str> {
    if let Some(last_slash_at) = url.rfind('/') {
        let (_, remote_project_name) = url.split_at(last_slash_at+1);
        return Some(remote_project_name);
    }
    None
}

#[test]
fn test_url_to_projectname() {
    assert_eq!(url_to_projectname("http://das/haus/vom/nikolause"), Some("nikolause"));
    assert_eq!(url_to_projectname("http://."), Some("."));
    assert_eq!(url_to_projectname("nikolause"), None);
    assert_eq!(url_to_projectname("n/i/k/o/lause"), Some("lause"));
    assert_eq!(url_to_projectname(""), None);
}
