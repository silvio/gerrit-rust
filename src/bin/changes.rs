
//! manage endpoint `/changes/`

use clap::{self, App, SubCommand, Arg};
use gerritlib::error::GGRError;
use gerritlib::error::GGRResult;
use gerritlib::gerrit::Gerrit;
use config;

/// returns the *Changes* part of gerrit-rusts menu
pub fn menu<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("changes")
    .about("changes management")
    .subcommand(SubCommand::with_name("query")
                .about("queries changes")
                .arg(Arg::with_name("regexp-selector")
                     .help("select fields to print; via regular expression")
                     .takes_value(true)
                     .conflicts_with("simple-selector")
                     .long("regexp-selector")
                     .multiple(true)
                     .short("f")
                )
                .arg(Arg::with_name("ofields")
                     .help("return optional fields information")
                     .short("o")
                     .takes_value(true)
                )
                .arg(Arg::with_name("userquery")
                     .help("user query for changes")
                     .required(true)
                     .multiple(true)
                     .takes_value(true)
                )
                .arg(Arg::with_name("fieldslist")
                     .help("get all fields useable for --fields options")
                     .short("l")
                )
                .arg(Arg::with_name("raw")
                     .help("print machine readable raw json stream, useful for \
                            pretty printer. `--fields` and `--fieldslist` are \
                            ignored.")
                     .short("r")
                     .conflicts_with("human")
                )
                .arg(Arg::with_name("human")
                     .help("print human readable json stream `--fields` and \
                            `--fieldslist` are ignored.")
                     .short("u")
                     .conflicts_with("raw")
                )
        )
}

/// proxy function of implemented features
///
/// Currently implemented sub commands:
///
/// * query
pub fn manage(x: &clap::ArgMatches, config: config::Config) -> GGRResult<()> {
    match x.subcommand() {
        ("query", Some(y)) => { query(y, config) },
        _ => {
            println!("{}", x.usage());
            Ok(())
        },
    }
}

/// create, call and prints queries to a gerrit server
fn query(y: &clap::ArgMatches, config: config::Config) -> GGRResult<()> {
    let userquery = match y.values_of_lossy("userquery") {
        Some(x) => Query::from(x),
        None => return Err(GGRError::General("No or bad userquery".into())),
    };

    let regsel = match y.values_of_lossy("regexp-selector") {
        Some(b) => b,
        None => vec!(String::from(".*")),
    };

    let fieldslist = y.is_present("fieldslist");
    let raw = y.is_present("raw");
    let human = y.is_present("human");
    let ofields  = y.values_of_lossy("ofields");

    let mut gerrit = Gerrit::new(config.get_base_url());

    let changeinfos = try!(gerrit.changes(Some(userquery.get_query()), ofields, config.get_username(), config.get_password()));

    if raw {
        println!("{}", changeinfos.raw());
        return Ok(());
    }

    if human {
        println!("{}", changeinfos.human());
        return Ok(());
    }

    if fieldslist {
        let (count, hm) = changeinfos.fieldslist();
        let mut printout = String::new();

        let mut vec_hm: Vec<(&String, &usize)> = hm.iter().collect();
        vec_hm.sort();

        for entry in vec_hm {
            printout.push_str(&format!("{}({})", entry.0, entry.1));
            printout.push(' ');
        }
        println!("{} -> {}", count, printout);
    } else {
        println!("{}", changeinfos.as_string_reg(&regsel).trim());
    }

    Ok(())
}

#[derive(Clone)]
struct Query {
    query: Vec<String>,
}

impl From<Vec<String>> for Query {
    fn from(v: Vec<String>) -> Query {
        let mut qb = Query::new();

        for arg in v {
            qb.add_str(arg);
        }
        qb
    }
}

impl Query {
    pub fn new() -> Query {
        Query {
            query: Vec::new()
        }
    }

    /// Split at first ':' from left so we can have ':' in search string
    pub fn add_str(&mut self, x: String) -> &mut Query {
        // TODO: add preparsing of `x` to prevent missuse like `x=y` instead of `x:y`.
        self.query.push(x);
        self
    }

    pub fn get_query(&self) -> &Vec<String> {
        &self.query
    }
}
