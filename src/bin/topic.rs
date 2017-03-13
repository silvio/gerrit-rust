
use clap::{self, SubCommand, App, Arg};
use git2::Repository;
use git2::BranchType;
use std::collections::HashMap;
use std::error::Error;
use std::io::{self, Write};
use std::process::Command;
use libgerrit::error::GGRError;
use libgerrit::error::GGRResult;
use libgerrit::gerrit::Gerrit;
use libgerrit::entities;
use config;

pub fn menu<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("topic")
    .about("topic management")
    .subcommand(SubCommand::with_name("create")
                .about("Create topic branch")
                .arg(Arg::with_name("branchname")
                     .help("branch name to create")
                     .required(true)
                     .index(1)
                )
                .arg(Arg::with_name("repo")
                     .short("r")
                     .long("repo")
                     .help("Create topic branch on this repository. \
                            Use <repo>[:<git-reference>] to point to a specific repository. \
                            Current repository is '.' \
                            '<git-reference>' defaults to HEAD. \
                            Example: -r .:origin/master -r test -r project:4d6d711")
                     .next_line_help(true)
                     .required(true)
                     .multiple(true)
                     .takes_value(true)
                )
    )
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
        ("create", Some(y)) => { create(y) },
        ("forget", Some(y)) => { forget(y) },
        ("fetch", Some(y)) => { fetch(y, config) },
        ("checkout", Some(y)) => { checkout(y, config) },
        ("reviewer", Some(y)) => { reviewer(y, config) },
        _ => {
            println!("{}", x.usage());
            Ok(())
        },
    }
}

/// Create topics
fn create(y: &clap::ArgMatches) -> GGRResult<()> {
    let branchname = match y.value_of("branchname") {
        Some(x) => x,
        None => return Err(GGRError::General("Bad branchname".into())),
    };

    let repository_names = match y.values_of_lossy("repo") {
        Some(x) => x,
        None => return Err(GGRError::General("Bad Reponame".into())),
    };

    let mut repo;
    if repository_names.is_empty() {
        println!("Create topic branch \"{}\" at repository:", branchname);
        for subrep in repository_names {
            let (repo_name, reference_name) = split_repo_reference(&subrep);

            print!("* {}: ", &repo_name);
            repo = try!(Repository::open(&repo_name));
            match repo.revparse_single(&reference_name) {
                Ok(object) => {
                    let commit = match object.as_commit() {
                        Some(c) => c,
                        None => { println!("fail (not exists)"); continue},
                    };

                    match repo.branch(branchname, commit, false) {
                        Ok(_) => { println!("done") },
                        Err(e) => { println!("fail ({})", e.message()); },
                    };
                },
                Err(e) => {
                    println!("fail ({})", e.message())
                },
            }
        }
    }

    Ok(())
}

/// delete topics
fn forget(y: &clap::ArgMatches) -> GGRResult<()> {
    let branchname = match y.value_of("branchname") {
        Some(x) => x,
        None => return Err(GGRError::General("Bad branchname".into())),
    };

    let repo = try!(Repository::discover("."));

    /* remove branch on the current repository */
    match repo.find_branch(branchname, BranchType::Local) {
        Ok(mut branch) => {
            print!("* current folder: ");
            if branch.delete().is_err() {
                println!("fail");
            } else {
                println!("done");
            }
        },
        Err(err) => {
            return Err(GGRError::from(err));
        }
    };

    /* remove branch on all submodules */
    if y.is_present("recursive") {
        let submodules = try!(repo.submodules());
        for sm in &submodules {
            let reposub = try!(sm.open());
            match reposub.find_branch(branchname, BranchType::Local) {
                Ok(mut branch) => {
                    print!("* {}: ", sm.path().display());
                    if branch.delete().is_err() {
                        println!("fail");
                    } else {
                        println!("done");
                    }
                },
                Err(err) => {
                    return Err(GGRError::from(err));
                }
            };
        }
    }
    Ok(())
}

/// splits a string to repository and reference
///
/// 't' can have this possible cases and output:
///
/// * 'a':      (repo=a, reference=HEAD)
/// * 'a:b'     (repo=a, reference=b)
/// * 'a:b:c'   (repo=a, reference=b)
fn split_repo_reference(t: &str) -> (String, String) {
    let repo;
    let reference;

    let mut splited = t.split(':');

    if splited.clone().count() >= 2 {
        // unwrap are save in this context
        repo = String::from(splited.next().unwrap());
        reference = String::from(splited.next().unwrap());
    } else {
        reference = String::from("HEAD");
        repo = t.to_owned().to_string();
    }

    (repo, reference)
}

#[test]
fn test_split_repo_reference() {
    assert_eq!(split_repo_reference("a"), ("a".to_string(),"HEAD".to_string()));
    assert_eq!(split_repo_reference("a:b"), ("a".to_string(),"b".to_string()));
    assert_eq!(split_repo_reference("a:b:c"), ("a".to_string(),"b".to_string()));
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

    let mut gerrit = Gerrit::new(config.get_base_url());
    fetch_topic(&mut gerrit, topicname, local_branch_name, force, tracking_branch_name, closed)
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
    if let Ok(cis) = gerrit.changes().add_query_part(format!("topic:{}", topicname)).query_changes() {

        // manipulate reviewer for topic
        if let Some(ref reviewerlist) = y.values_of_lossy("reviewers") {
            for ci in cis {
                for reviewer in reviewerlist {
                    let remove = reviewer.starts_with('~');

                    if remove {
                        let reviewer = &reviewer[1..];
                        let id = match ci {
                            entities::ChangeInfo::Gerrit0209(ref x) => {
                                &x.id
                            },
                            entities::ChangeInfo::Gerrit0213(ref x) => {
                                &x.id
                            },
                        };
                        if let Err(res) = gerrit.changes().delete_reviewer(&id, reviewer) {
                            match res {
                                GGRError::GerritApiError(ref x) => {
                                    println!("{}, {}", reviewer, x.description());
                                },
                                x => { println!("Other error: {:?}", x);}
                            };
                        } else {
                            println!("{}: removed", reviewer);
                        };
                    } else {
                        let id = match ci {
                            entities::ChangeInfo::Gerrit0209(ref x) => {
                                &x.id
                            },
                            entities::ChangeInfo::Gerrit0213(ref x) => {
                                &x.id
                            },
                        };
                        match gerrit.changes().add_reviewer(&id, reviewer) {
                            Ok(addreviewerresult) => {
                                match addreviewerresult {
                                    entities::AddReviewerResult::Gerrit0209(g) => {
                                        match g.reviewers {
                                            Some(reviewerret) => {
                                                for r in reviewerret {
                                                    println!("{}, {}, {}: added",
                                                             r.name.unwrap_or("unkown name".into()),
                                                             r.email.unwrap_or("unkown mail".into()),
                                                             r._account_id.unwrap_or(99999999));
                                                }
                                            },
                                            None => {
                                                println!("Not added: {}", g.error.unwrap_or("No error message from gerrit server provided".into()));
                                            },
                                        }
                                    },
                                    entities::AddReviewerResult::Gerrit0213(g) => {
                                        match g.reviewers {
                                            Some(reviewerret) => {
                                                for r in reviewerret {
                                                    println!("{}, {}, {}: added",
                                                             r.name.unwrap_or("unkown name".into()),
                                                             r.email.unwrap_or("unkown mail".into()),
                                                             r._account_id.unwrap_or(99999999));
                                                }
                                            },
                                            None => {
                                                println!("Not added: {}", g.error.unwrap_or("No error message from gerrit server provided".into()));
                                            },
                                        }
                                    },
                                };
                            },
                            Err(e) => {
                                println!("Problem to add '{}' as reviewer: {}", reviewer, e);
                            },
                        }
                    }
                }
            }

            return Ok(());
        }

        // only list reviewers
        for ci in cis {
            let (id, subject) = match ci {
                entities::ChangeInfo::Gerrit0209(ref x) => {
                    (&x.id, &x.subject)
                },
                entities::ChangeInfo::Gerrit0213(ref x) => {
                    (&x.id, &x.subject)
                },
            };

            println!("* reviewer for {}:", subject);
            if let Ok(reviewers) = gerrit.changes().get_reviewers(&id) {
                let mut reviewer_list = Vec::new();
                for reviewer in reviewers {
                    let (name, username, email, approval) = match reviewer {
                        entities::ReviewerInfo::Gerrit0209(g) => {
                            (g.name.unwrap_or_else(|| "unknown".into()), g.username.unwrap_or_else(|| "unknown".into()), g.email.unwrap_or_else(|| "unknown".into()), g.approvals)
                        },
                        entities::ReviewerInfo::Gerrit0213(g) => {
                            (g.name.unwrap_or_else(|| "unknown".into()), g.username.unwrap_or_else(|| "unknown".into()), g.email.unwrap_or_else(|| "unknown".into()), g.approvals)
                        },
                    };

                    reviewer_list.push(name.clone());

                    if verbose {
                        println!("  * {:2}/{:2} {:15.15} {:15.15} {}",
                                 approval.verified.unwrap_or(0), approval.codereview.unwrap_or(0),
                                 name, username, email);
                    }
                }
                if ! verbose {
                    print!("  ");
                    for reviewer in reviewer_list {
                        print!("{}, ", reviewer);
                    }
                    println!("");
                }
            }
        }
    } else {
        println!("no changes for '{}' found", topicname);
    }

    Ok(())
}

/// Conviention function to fetch topic `topicname` to branch `local_branch_name`.
///
/// If branch exists and `force` is true, the branch is moving to new position.
fn fetch_topic(gerrit: &mut Gerrit, topicname: &str, local_branch_name: &str, force: bool, tracking_branch_name: Option<&str>, closed: bool) -> GGRResult<()> {
    let mut changes = gerrit.changes();
    changes.add_label("CURRENT_REVISION").add_label("CURRENT_COMMIT");
    changes.add_query_part(format!("topic:{}", topicname));
    if !closed {
        changes.add_query_part("status:open");
    }

    if let Ok(changeinfos) = changes.query_changes() {
        let project_tip = project_tip(&changeinfos).unwrap();

        // try to fetch topic for main_repo and all submodules
        'next_ptip: for (p_name, p_tip) in project_tip {
            print!("fetch {} for {} ... ", p_name, p_tip);
            // check for root repository
            if let Ok(main_repo) = Repository::open(".") {
                // check changes on root repository
                match fetch_from_repo(&main_repo, &changeinfos, force, local_branch_name, &p_name, &p_tip, tracking_branch_name) {
                    Ok((true,m)) => {
                        println!("{}", m);
                        continue;
                    },
                    Ok((false, m)) => {
                        println!("KO\n  Error: {}", m.trim());
                    },
                    Err(r) => {
                        // hide all other errors
                        let r = r.to_string();
                        if !r.is_empty() {
                            println!("KO\nError: {}", r.to_string().trim());
                        }
                    }
                };

                // check for submodules
                if let Ok(smodules) = main_repo.submodules() {
                    for smodule in smodules {
                        if let Ok(sub_repo) = smodule.open() {
                            match fetch_from_repo(&sub_repo, &changeinfos, force, local_branch_name, &p_name, &p_tip, tracking_branch_name) {
                                Ok((true, m)) => {
                                    println!("{}", m);
                                    continue 'next_ptip;
                                },
                                Ok((false, m)) => {
                                    println!("KO\n  Error: {}", m.trim());
                                    continue;
                                },
                                Err(r) => {
                                    let r = r.to_string();
                                    if !r.is_empty() {
                                                println!("KO\nError: {}", r.to_string().trim());
                                    }
                                }
                            }
                        } else {
                            println!("{} not opened", smodule.name().unwrap());
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

/// convenient function to pull a `p_tip` from a `repo`, if `basename(repo.url)` same as `p_name`
/// is.
///
/// returns `true` if something is pulled, and `false` if no pull was executed. The String object
/// is a status message.
fn fetch_from_repo(repo: &Repository, ci: &[entities::ChangeInfo], force: bool, local_branch_name: &str, p_name: &str, p_tip: &str, tracking_branch_name: Option<&str>) -> GGRResult<(bool, String)> {
    if repo.is_bare() {
        return Err(GGRError::General(format!("repository path '{:?}' is bare, we need a workdir", repo.path())));
    }

    for remote_name in repo.remotes().unwrap().iter() {
        if let Ok(remote) = repo.find_remote(remote_name.unwrap()) {
            let url = remote.url().unwrap().to_owned();
            let check_project_names = vec!(
                p_name.into(),
                format!("{}.git", p_name)
            );



            if check_project_names.contains(&String::from(url_to_projectname(&url).unwrap())) {
                let entity = entity_from_commit(ci, p_tip)?;

                let reference = match *entity {
                    entities::ChangeInfo::Gerrit0209(ref x) => {
                        if let Some(ref cur_rev) = x.current_revision {
                            if let Some(ref revisions) = x.revisions {
                                if let Some(ref current_revision) = revisions.get(cur_rev) {
                                    if let Some(ref fetchref) = current_revision.fetch.get("http") {
                                        &fetchref.reference
                                    } else {
                                        return Err(GGRError::General("No fetch ref".into()));
                                    }
                                } else {
                                    return Err(GGRError::General("no current revisions".into()));
                                }
                            } else {
                                return Err(GGRError::General("No revisions".into()));
                            }
                        } else {
                            return Err(GGRError::General("No cur_rev".into()));
                        }
                    },
                    entities::ChangeInfo::Gerrit0213(ref x) => {
                        if let Some(ref cur_rev) = x.current_revision {
                            if let Some(ref revisions) = x.revisions {
                                if let Some(ref current_revision) = revisions.get(cur_rev) {
                                    if let Some(ref fetchref) = current_revision.fetch.get("http") {
                                        &fetchref.reference
                                    } else {
                                        return Err(GGRError::General("No fetch ref".into()));
                                    }
                                } else {
                                    return Err(GGRError::General("no current revisions".into()));
                                }
                            } else {
                                return Err(GGRError::General("No revisions".into()));
                            }
                        } else {
                            return Err(GGRError::General("No cur_rev".into()));
                        }
                    },
                };


                let force_string = if force {"+"} else { "" };
                let refspec = format!("{}{}:{}", force_string, reference, local_branch_name);

                if !force  && repo.find_branch(local_branch_name, BranchType::Local).is_ok() {
                    // Branch exists, but no force
                    return Ok((false, String::from("Branch exists and no force")));
                }

                let mut output_fetch = try!(Command::new("git")
                    .current_dir(repo.path())
                    .arg("fetch")
                    .arg(remote.name().unwrap())
                    .arg(refspec)
                    .output());

                if output_fetch.status.success() {
                    if let Some(tracking_branch) = tracking_branch_name {
                        let mut output_tracking = try!(Command::new("git")
                             .current_dir(repo.path())
                             .arg("branch")
                             .arg("--set-upstream-to")
                             .arg(tracking_branch)
                             .arg(local_branch_name)
                             .output());
                        if !output_tracking.stdout.is_empty() {
                            output_fetch.stdout.append(&mut String::from("\n* ").into_bytes());
                            output_fetch.stdout.append(&mut output_tracking.stdout);
                        }
                        if !output_tracking.stderr.is_empty() {
                            output_fetch.stdout.append(&mut String::from("\n* ").into_bytes());
                            output_fetch.stdout.append(&mut output_tracking.stderr);
                        }
                    }

                    return Ok((true, try!(String::from_utf8(output_fetch.stdout))));
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
        let project = match *element {
            entities::ChangeInfo::Gerrit0209(ref x) => {
                &x.project
            },
            entities::ChangeInfo::Gerrit0213(ref x) => {
                &x.project
            },
        };
        if !list_of_projects.contains(project) {
            list_of_projects.push(project.clone());
        }
    }

    // find tip of every project
    let mut project_tip: HashMap<String, String> = HashMap::new();
    for project in list_of_projects {
        // find in entities the last change of every project for this topic
        let mut list_all_parents = Vec::new();
        // fill a list with all parents
        for element in changes {
            match *element {
                entities::ChangeInfo::Gerrit0209(ref element) => {
                    if let Some(ref cur_revision) = element.current_revision {
                        if let Some(ref revisions) = element.revisions {
                            if let Some(ref cur_revision) = revisions.get(cur_revision) {
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
                },
                entities::ChangeInfo::Gerrit0213(ref element) => {
                    if let Some(ref cur_revision) = element.current_revision {
                        if let Some(ref revisions) = element.revisions {
                            if let Some(ref cur_revision) = revisions.get(cur_revision) {
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
                },
            };
        }

        for element in changes {
            match *element {
                entities::ChangeInfo::Gerrit0209(ref element) => {
                    if element.project == *project {
                        if let Some(ref cur_revision) = element.current_revision {
                            if !list_all_parents.contains(&cur_revision) {
                                // a tip commit is never a parent for a topic
                                project_tip.insert(project, cur_revision.clone());
                                break;
                            }
                        }
                    }
                },
                entities::ChangeInfo::Gerrit0213(ref element) => {
                    if element.project == *project {
                        if let Some(ref cur_revision) = element.current_revision {
                            if !list_all_parents.contains(&cur_revision) {
                                // a tip commit is never a parent for a topic
                                project_tip.insert(project, cur_revision.clone());
                                break;
                            }
                        }
                    }
                },
            };
        }
    }

    Ok(project_tip)
}

pub fn entity_from_commit<'ci>(changes: &'ci [entities::ChangeInfo], commit: &str) -> GGRResult<&'ci entities::ChangeInfo> {
    for element in changes {
        match *element {
            entities::ChangeInfo::Gerrit0209(ref x) => {
                if let Some(ref revisions) = x.revisions {
                    for rev in revisions.keys() {
                        if rev == commit {
                            return Ok(element);
                        }
                    }
                }
            },
            entities::ChangeInfo::Gerrit0213(ref x) => {
                if let Some(ref revisions) = x.revisions {
                    for rev in revisions.keys() {
                        if rev == commit {
                            return Ok(element);
                        }
                    }
                }
            },
        };
    }

    Err(GGRError::General("no entity found".into()))
}


/// Convenient function to checkout a topic
pub fn checkout_topic(branchname: &str) -> GGRResult<()> {
        if let Ok(main_repo) = Repository::open(".") {
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
fn checkout_repo(repo: &Repository, branchname: &str) -> GGRResult<()> {
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

