
use clap::{self, SubCommand, App, Arg};
use libgerrit::error::GGRResult;
use libgerrit::gerrit::Gerrit;
use config;
use libgerrit::entities;

pub fn menu<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("gerritapi")
    .about("Gerrit API interface (Only fo API tests)")
    .subcommand(SubCommand::with_name("changes")
                .about("Change endpoint")
                .subcommand(SubCommand::with_name("create")
                            .about("Create a change")
                            .arg(Arg::with_name("project")
                                 .required(true)
                                 .takes_value(true)
                                 .long("project")
                                 .short("p")
                                 .help("The name of the project")
                            )
                            .arg(Arg::with_name("branch")
                                 .required(true)
                                 .takes_value(true)
                                 .long("branch")
                                 .short("b")
                                 .help("The name of the target branch. The 'refs/heads/' prefix is omitted.")
                             )
                            .arg(Arg::with_name("subject")
                                 .required(true)
                                 .takes_value(true)
                                 .long("subject")
                                 .short("s")
                                 .help("The subject of the change (header line of the commit message).")
                             )
                )
                .subcommand(SubCommand::with_name("query")
                            .about("query changes")
                            .arg(Arg::with_name("query")
                                 .required(true)
                                 .takes_value(true)
                                 .long("query")
                                 .short("q")
                                 .help("Query string")
                             )
                )
    )
}

pub fn manage(x: &clap::ArgMatches, config: config::Config) -> GGRResult<()> {
    match x.subcommand() {
        ("changes", Some(y)) => { changes(y, config) },
        _ => {
            println!("{}", x.usage());
            Ok(())
        },
    }
}

fn changes(y: &clap::ArgMatches, config: config::Config) -> GGRResult<()> {
    let mut gerrit = Gerrit::new(config.get_base_url());

    match y.subcommand() {
        ("create", Some(opt)) => {
            let project = opt.value_of("project").unwrap().into();
            let branch = opt.value_of("branch").unwrap().into();
            let subject = opt.value_of("subject").unwrap().into();

            let ci = entities::ChangeInput {
                project: project,
                branch: branch,
                subject: subject,
                base_change: None,
                merge: None,
                new_branch: None,
                status: None,
                topic: None,
            };

            match gerrit.changes().create_change(&ci) {
                Ok(changeinfo) => {
                    println!("Change created! Returned data");
                    println!("{:?}", changeinfo);
                },
                Err(x) => {
                    println!("Error: {:?}", x);
                }
            }
        },

        ("query", Some(opt)) => {
            let query = opt.value_of("query").unwrap();

            match gerrit.changes().add_query_part(query).query_changes() {
                Ok(cis) => {
                    for i in cis {
                        println!("* {:?}", i);
                    }
                },
                Err(x) => {
                    println!("Error: {:?}", x);
                }
            }
        },

        e => {
            println!("unknown subcommand {}", e.0);
            println!("{}", y.usage());
        }
    }

    Ok(())
}
