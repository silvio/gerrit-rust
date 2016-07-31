
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
pub fn manage(x: &clap::ArgMatches) -> GGRResult<()> {
    match x.subcommand() {
        ("query", Some(y)) => { query(y) },
        _ => {
            println!("{}", x.usage());
            Ok(())
        },
    }
}

/// creat, call and prints queries to a gerrit server
fn query(y: &clap::ArgMatches) -> GGRResult<()> {
    let mut userquery = match y.values_of("userquery") {
        Some(x) => Query::from(x),
        None => return Err(GGRError::General("No or bad userquery".into())),
    };

    let configfile = try!(config::ConfigFile::discover(".", ".ggr.conf"));
    let config = config::Config::from_configfile(configfile);

    if ! config.is_valid() {
        return Err(GGRError::General("problem with configfile".to_string()));
    }

    let gerrit = Gerrit::new(config.get_base_url());

    let response_changes = gerrit.changes(Some(userquery.get_query()), None);

    match response_changes {
        Ok(changeinfos) => {
            for ci in changeinfos {
                println!("{}", ci);
            }
        },
        Err(x) => {
            return Err(x);
        }
    }

    Ok(())
}

#[derive(Clone)]
struct Query<'query> {
    query: Vec<&'query str>,
}

impl<'value> From<clap::Values<'value>> for Query<'value> {
    fn from(v: clap::Values<'value>) -> Query<'value> {
        let mut qb = Query::new();

        for (_, arg) in v.enumerate() {
            qb.add_str(arg);
        }
        qb
    }
}

impl<'query> Query<'query> {
    pub fn new() -> Query<'query> {
        Query {
            query: Vec::new()
        }
    }

    /// Split at first ':' from left so we can have ':' in search string
    pub fn add_str(&mut self, x: &'query str) -> &mut Query<'query> {
        // TODO: add preparsing of `x` to prevent missuse like `x=y` instead of `x:y`.
        self.query.push(x);
        self
    }

    pub fn get_query(&self) -> &Vec<&'query str> {
        &self.query
    }
}
