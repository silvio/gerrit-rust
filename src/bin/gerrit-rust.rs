
//! main entry for `gerrit-rust`

extern crate chrono;
extern crate clap;
extern crate env_logger;
extern crate gerritlib;
extern crate git2;
#[macro_use]
extern crate log;
extern crate rustc_serialize;
extern crate toml_config;

pub mod changes;
pub mod config;
pub mod topic;

use clap::App;
use gerritlib::error::GGRError;
use std::error::Error;
use std::process::exit;

fn init_log() {
    let format = |record: &log::LogRecord| {
        format!("[{:5.5}] [{}] [{}] - {}", record.level(),
                chrono::Local::now().to_rfc3339(),
                record.location().module_path(),
                record.args())
    };

    let mut builder = env_logger::LogBuilder::new();
    builder.format(format).filter(None, log::LogLevelFilter::Info);

    if let Ok(ref rl) = std::env::var("RUST_LOG") {
        builder.parse(rl);
    }

    let _ = builder.init();
}

fn main() {
    init_log();

    let mut app = App::new("gerrit-rust")
        .author("Silvio Fricke <silvio.fricke@gmail.com>")
        .about("some gerrit tools")
        .subcommand(topic::menu())
        .subcommand(changes::menu())
        .subcommand(config::menu())
        ;

    let matches = app.clone().get_matches();

    let configfile = match config::ConfigFile::discover(".", ".ggr.conf") {
        Ok(c) => c,
        Err(x) => {
            println!("Problem with loading of config file:");
            println!("{}", x.to_string());
            exit(-1);
        },
    };
    let config = config::Config::from_configfile(configfile);
    if ! config.is_valid() {
        panic!(GGRError::General("problem with configfile".to_string()));
    }

    let out = match matches.subcommand() {
        ("topic", Some(x)) => { topic::manage(x, config) },
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
