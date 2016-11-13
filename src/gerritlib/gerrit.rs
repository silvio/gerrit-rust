
//! Implements the gerrit structure

use call::Call;
use changes;
use error::GGRResult;
use error::GGRError;
use git2::Repository;
use git2::BranchType;
use git2::Oid;


/// `Gerrit` structure for management of several gerrit endpoints
pub struct Gerrit {
    call: Call,
}

impl Gerrit {
    /// Creates a new `Gerrit` object
    ///
    /// The url points to the http endpoint of an gerrit server like
    /// `http://localhost:8080/gerrit`. All other function append to this url there endpoint pathes
    /// and query parameters.
    pub fn new<S>(url: S) -> Gerrit
    where S: Into<String> {
        Gerrit {
            call: Call::new(url.into()),
        }
    }

    /// query changes from gerrit server
    ///
    /// `querylist` is used as filter for the call to gerrit. `additional_infos` gives some more
    /// information of one Change entity.
    pub fn changes(&mut self, querylist: Option<&Vec<String>>, additional_infos: Option<Vec<String>>, username: &str, password: &str)
        -> GGRResult<changes::ChangeInfos>
    {
        let mut querystring = "pp=0&q=".to_string();
        match querylist {
            None => { /* nothing to do, we call without filter */ },
            Some(x) => {
                let urlfragment = Changes::build_url(x);
                querystring = format!("{}{}", querystring, urlfragment);
            },
        };

        if let Some(labels) = additional_infos {
            if !labels.is_empty() {
                for label in labels {
                    querystring = format!("{}&o={}", querystring, label);
                }
            }
        }

        if !username.is_empty() && !password.is_empty() {
            self.call.set_credentials(username, password);
        }

        changes::Changes::query_changes(&self.call, &querystring)
    }

    /// fetch topic `topicname` to branch `local_branch_name`.
    ///
    /// If branch exists and `force` is true, the branch is moving to new position.
    pub fn fetch_topic(&mut self, topicname: &str, local_branch_name: &str, force: bool, username: &str, password: &str) -> GGRResult<()> {
        let ofields: Vec<String> = vec!("CURRENT_REVISION".into(), "CURRENT_COMMIT".into());

        let changeinfos = try!(self.changes(Some(&vec![format!("topic:{} status:open", topicname)]), Some(ofields), username, password));
        let project_tip = changeinfos.project_tip().unwrap();

        // TODO: remove some unwraps here
        // TODO: un-indent this part some more
        // try to fetch topic for main_repo and all submodules
        'next_ptip: for (p_name, p_tip) in project_tip {
            // check for root repository
            if let Ok(main_repo) = Repository::open(".") {
                match fetch_from_repo(&main_repo, &changeinfos, force, topicname, local_branch_name, &p_name, &p_tip) {
                    Ok((true,m)) => {
                        println!("OK ({})", m);
                        continue;
                    },
                    Ok((false, _)) => {
                    },
                    Err(r) => {
                        println!("Error: {}", r.to_string());
                    }
                };


                // check for submodules
                if let Ok(smodules) = main_repo.submodules() {
                    for smodule in smodules {
                        if let Ok(sub_repo) = smodule.open() {
                            match fetch_from_repo(&sub_repo, &changeinfos, force, topicname, local_branch_name, &p_name, &p_tip) {
                                Ok((true, m)) => {
                                    println!("OK ({})", m);
                                    continue 'next_ptip;
                                },
                                Ok((false, _)) => {
                                    continue 'next_ptip;
                                },
                                Err(r) => {
                                    println!("Error: {}", r.to_string());
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }

}

/// convience function to pull a `p_tip` from a `repo`, if `basename(repo.url)` same as `p_name`
/// is.
///
/// returns `true` if something is pulled, and `false` if no pull was executed. The String object
/// is a reflog message.
fn fetch_from_repo(repo: &Repository, ci: &changes::ChangeInfos, force: bool, topicname: &str, local_branch_name: &str, p_name: &str, p_tip: &str) -> GGRResult<(bool, String)> {
    for remote_name in repo.remotes().unwrap().iter() {
        if let Ok(mut remote) = repo.find_remote(remote_name.unwrap_or("")) {
            let url = remote.url().unwrap().to_owned();
            if url_to_projectname(&url).unwrap() == p_name {
                if let Ok(entity) = ci.entity_from_commit(p_tip) {
                    if let Some(ref cur_rev) = entity.current_revision {
                        if let Some(ref revisions) = entity.revisions {
                            let reference = &revisions.get(cur_rev).unwrap().reference;
                            let force_string = if force {"+"} else { "" };
                            let refspec = format!("{}{}", force_string, reference);
                            let reflog = format!("ggr: topic pull {}:\"{}\":{} ({}) commit:{}", p_name, topicname, reference, refspec, p_tip);

                            let ret = match remote.fetch(&[&refspec], None, Some(&reflog)) {
                                Err(r) => {
                                    Err(GGRError::from(r))
                                },
                                Ok(_) => {
                                    match Oid::from_str(p_tip) {
                                        Ok(oid) => {
                                            match repo.find_commit(oid) {
                                                Ok(commit) => {
                                                    // check branch exists and no force
                                                    let (real_branchname, real_force) = match repo.find_branch(local_branch_name, BranchType::Local) {
                                                        Ok(_) => {
                                                            if force {
                                                                (local_branch_name, force)
                                                            } else {
                                                                println!("branch '{}' exists, checkout to '{}'", local_branch_name, reference);
                                                                (reference.as_str(), true)
                                                            }
                                                        },
                                                        Err(_) => {
                                                            (local_branch_name, force)
                                                        }
                                                    };
                                                    match repo.branch(real_branchname, &commit, real_force) {
                                                        Ok(_) => Ok((true, reflog)),
                                                        Err(r) => {
                                                            Err(GGRError::from(r))
                                                        },
                                                    }
                                                },
                                                Err(r) => {
                                                    Err(GGRError::from(r))
                                                }
                                            }
                                        },
                                        Err(r) => {
                                            Err(GGRError::from(r))
                                        }
                                    }
                                },
                            };
                            return ret;
                        }
                    }
                }
            }
        }
    }

    Ok((false, String::from("")))
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

// helper structures
struct Changes;
impl Changes {
    pub fn build_url(querylist: &[String]) -> String {
        let mut out = String::new();
        for el in querylist.iter() {
            out.push_str(el);
            out.push_str("+");
        }
        if let Some(x) = out.chars().last() {
            if x == '+' {
                out = out.trim_right_matches(x).to_string();
            }
        };

        out
    }
}

#[test]
fn test_changes_build_url() {
    assert_eq!(Changes::build_url(&vec!()), "".to_string());
    assert_eq!(Changes::build_url(&vec!("a:1".to_string(), "b:2".to_string())), "a:1+b:2".to_string());
    assert_eq!(Changes::build_url(&vec!("a:1".to_string())), "a:1".to_string());
}
