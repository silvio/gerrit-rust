
//! manage endpoint `/changes/`

use clap;
use gerritlib::error::GGRError;
use gerritlib::error::GGRResult;
use gerritlib::gerrit::Gerrit;
use config;

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

/// creat, call and prints queries to a gerrit server
fn query(y: &clap::ArgMatches, config: config::Config) -> GGRResult<()> {
    let userquery = match y.values_of_lossy("userquery") {
        Some(x) => Query::from(x),
        None => return Err(GGRError::General("No or bad userquery".into())),
    };

    let fields = match y.values_of_lossy("fields") {
        Some(x) => x,
        None => return Err(GGRError::General("'fields' option wrong".into())),
    };

    let fieldslist = y.is_present("fieldslist");
    let raw = y.is_present("raw");
    let ofields  = y.values_of_lossy("ofields");

    let mut gerrit = Gerrit::new(config.get_base_url());

    let response_changes = gerrit.changes(Some(userquery.get_query()), ofields, config.get_username(), config.get_password());

    match response_changes {
        Ok(changeinfos) => {
            if raw {
                println!("{}", changeinfos.raw());
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
                println!("{}", changeinfos.as_string(&fields));
            }
        },
        Err(x) => {
            return Err(x);
        }
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
