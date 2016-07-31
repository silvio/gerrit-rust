
//! main entry for `gerrit-rust`

extern crate clap;
extern crate curl;
extern crate gerritlib;
extern crate git2;
extern crate rustc_serialize;
extern crate toml_config;

pub mod changes;
pub mod config;
pub mod topic;

use clap::Arg;
use clap::App;
use clap::SubCommand;
use std::error::Error;

fn main() {
    let mut app = App::new("gerrit-rust")
        .author("Silvio Fricke <silvio.fricke@gmail.com>")
        .about("some gerrit tools")
        .subcommand(SubCommand::with_name("topic")
                    .about("topic management")
                    .subcommand(SubCommand::with_name("create")
                                .help("Create topic branches")
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
                                .help("Delete topic branch")
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
        )
        .subcommand(SubCommand::with_name("changes")
                    .about("changes management")
                    .subcommand(SubCommand::with_name("query")
                                .help("queries changes")
                                .arg(Arg::with_name("fields")
                                     .help("select fields to print,\
                                            default is project,subject,topic")
                                     .short("f")
                                     .takes_value(true)
                                )
                                .arg(Arg::with_name("userquery")
                                     .help("user query for changes")
                                     .required(true)
                                     .multiple(true)
                                     .takes_value(true)
                                )
                    )
        )
        .subcommand(SubCommand::with_name("config")
                    .about("config management for ggr")
                    .subcommand(SubCommand::with_name("list")
                                .help("List all config options")
                    )
        )
        ;

    let matches = app.clone().get_matches();


    let out = match matches.subcommand() {
        ("topic", Some(x)) => { topic::manage(x) },
        ("changes", Some(x)) => { changes::manage(x) },
        ("config", Some(x)) => { config::manage(x) },
        _ => { let _ = app.print_help(); Ok(()) },
    };

    match out {
        Ok(_) => {},
        Err(e) => {
            println!("Error: {}", e.description())
        },
    }
}
