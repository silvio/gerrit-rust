
use clap;
use git2::Repository;
use git2::BranchType;
use gerritlib::error::GGRError;
use gerritlib::error::GGRResult;

/// manage subfunction of `topic` command
///
/// Currently implemented sub commands:
///
/// * create
/// * forget
pub fn manage(x: &clap::ArgMatches) -> GGRResult<()> {
    match x.subcommand() {
        ("create", Some(y)) => { create(y) },
        ("forget", Some(y)) => { forget(y) },
        _ => {
            println!("{}", x.usage());
            Ok(())
        },
    }
}

fn create(y: &clap::ArgMatches) -> GGRResult<()> {
    let branchname = y.value_of("branchname").unwrap();

    let repository_names = y.values_of("repo").unwrap();
    let mut repo;
    if repository_names.count() > 0 {
        println!("Create topic branch \"{}\" at repository:", branchname);
        for (_, subrep) in y.values_of("repo").unwrap().enumerate() {
            let (repo_name, reference_name) = split_repo_reference(subrep);

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

fn forget(y: &clap::ArgMatches) -> GGRResult<()> {
    let branchname = y.value_of("branchname").unwrap();

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

    let mut splited = t.split('.');

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

