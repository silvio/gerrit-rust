
//! main entry for `gerrit-rust`

extern crate clap;
extern crate gerritlib;
extern crate git2;
extern crate rustc_serialize;
extern crate toml_config;

pub mod changes;
pub mod config;
pub mod topic;

use clap::App;
use clap::SubCommand;
use gerritlib::error::GGRError;
use std::error::Error;

fn main() {
    let mut app = App::new("gerrit-rust")
        .author("Silvio Fricke <silvio.fricke@gmail.com>")
        .about("some gerrit tools")
        .subcommand(topic::menu())
        .subcommand(changes::menu())
        .subcommand(SubCommand::with_name("config")
                    .about("config management for ggr")
                    .subcommand(SubCommand::with_name("list")
                                .help("List all config options")
                    )
        )
        ;

    let matches = app.clone().get_matches();

    let configfile = config::ConfigFile::discover(".", ".ggr.conf").expect("ConfigFile problem");
    let config = config::Config::from_configfile(configfile);
    if ! config.is_valid() {
        panic!(GGRError::General("problem with configfile".to_string()));
    }

    let out = match matches.subcommand() {
        ("topic", Some(x)) => { topic::manage(x) },
        ("changes", Some(x)) => { changes::manage(x, config) },
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
