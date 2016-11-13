
use clap::{self, SubCommand, App, Arg};
use git2::Repository;
use git2::BranchType;
use gerritlib::error::GGRError;
use gerritlib::error::GGRResult;
use gerritlib::gerrit::Gerrit;
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
    .subcommand(SubCommand::with_name("pull")
                .about("Pull a topic on current and all sub repositories")
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
    )
    .subcommand(SubCommand::with_name("checkout")
                .about("Checkout a branch on current and all sub repositories")
                .arg(Arg::with_name("branchname")
                     .help("local branch to checkout")
                     .required(true)
                     .takes_value(true)
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
pub fn manage(x: &clap::ArgMatches, config: config::Config) -> GGRResult<()> {
    match x.subcommand() {
        ("create", Some(y)) => { create(y) },
        ("forget", Some(y)) => { forget(y) },
        ("pull", Some(y)) => { pull(y, config) },
        ("checkout", Some(y)) => { checkout(y, config) },
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
fn fetch(y: &clap::ArgMatches, config: config::Config) -> GGRResult<()> {
    if !config.is_root() {
        return Err(GGRError::General("You have to run topic::fetch on the main/root repository".into()));
    }

    let topicname = y.value_of("topicname").expect("no or bad topicname");
    let force = y.is_present("force");
    let local_branch_name = y.value_of("branchname").unwrap_or(topicname);

    let mut gerrit = Gerrit::new(config.get_base_url());
    gerrit.fetch_topic(topicname, local_branch_name, force, config.get_username(), config.get_password())
}

/// checkout topics
fn checkout(y: &clap::ArgMatches, config: config::Config) -> GGRResult<()> {
    if !config.is_root() {
        return Err(GGRError::General("You have to run topic::checkout on the main/root repository".into()));
    }

    let branchname = y.value_of("branchname").unwrap();

    let mut gerrit = Gerrit::new(config.get_base_url());
    gerrit.checkout_topic(branchname)
}

