
//! manage endpoint `/changes/`

use clap::{self, App, SubCommand, Arg};
use libgerrit::error::GGRError;
use libgerrit::error::GGRResult;
use libgerrit::gerrit::Gerrit;
use libgerrit::entities;
use config;
use gron::ToGron;
use serde_json;
use regex;
use std::collections::HashMap;

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
                     .multiple(true)
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
    let mut gerrit = Gerrit::new(config.get_base_url());
    let mut changes = gerrit.changes();

    if let Some(userquery) = y.values_of_lossy("userquery") {
        for arg in userquery { changes.add_query_part(arg); }
    } else {
        return Err(GGRError::General("No or bad userquery".into()));
    };

    let regsel = match y.values_of_lossy("regexp-selector") {
        Some(x) => x,
        None => vec!(String::from(".*")),
    };
    let fieldslist = y.is_present("fieldslist");
    let raw = y.is_present("raw");
    let human = y.is_present("human");

    if let Some(ofields) = y.values_of_lossy("ofields") {
        for arg in ofields {
            changes.add_label(arg);
        }
    };

    match changes.query_changes() {
        Ok(cis) => {
            let changeinfos = ChangeInfos::new(cis);

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
        },
        Err(x) => {
            return Err(x);
        }
    }


    Ok(())
}

#[derive(Default, Debug)]
pub struct ChangeInfos {
    vec: Vec<entities::ChangeInfo>,
    filter_key: Vec<String>,
    filter_val: Vec<String>,
}

impl ChangeInfos {
    /// creates new ChangeInfos object with an initial ChangeInfos.json value
    pub fn new(init: Vec<entities::ChangeInfo>)
    -> ChangeInfos {
        ChangeInfos {
            vec: init,
            filter_key: Vec::new(),
            filter_val: Vec::new(),
        }
    }

    pub fn to_json(&self) -> serde_json::Value {
        serde_json::to_value(&self.vec).expect("Problem with to_value")
    }

    /// add a regular expression filter for keys
    ///
    /// The filter needs to be resetted through `filter_reset`.
    pub fn filter_key(&mut self, r: &str) -> &mut Self {
        self.filter_key.push(String::from(r));
        self
    }

    /// add a regular expression filter for values
    ///
    /// The filter needs to be resetted through `filter_reset`.
    pub fn filter_val(&mut self, r: String) -> &mut Self {
        self.filter_val.push(r);
        self
    }

    /// reset key and value filter
    pub fn filter_reset(&mut self) -> &mut Self {
        self.filter_val.clear();
        self.filter_key.clear();
        self
    }

    pub fn as_string_reg(&self, selectors: &[String]) -> String {
        let json = self.to_json();
        let mut grondata: Vec<u8> = Vec::new();
        let _ = json.to_gron(&mut grondata, "");
        let mut out = String::from("");

        for line in String::from_utf8(grondata).unwrap_or_default().lines() {
            let mut keyval = line.splitn(2, '=');
            let key = keyval.next().unwrap_or("").trim();
            let val = keyval.next().unwrap_or("").trim();

            // remove empty lines
            if key.is_empty() { continue };
            if val == "null" { continue };

            for selector in selectors {
                if let Ok(re) = regex::Regex::new(selector) {
                    if re.is_match(key) {
                        out.push_str(&format!("{} {}\n", key, val));
                    }
                }
            }
        }

        out
    }

    /// prints all selectable fields os a search string
    ///
    /// returns two values. First one is the count of returned json objects and second value is a
    /// HashMap<String, usize> with all fields and gow much they occure.
    pub fn fieldslist(&self) -> (usize, HashMap<String, usize>) {
        let mut out_hmap: HashMap<String, usize> = HashMap::new();
        let mut entries = 0;

            if let Some(array) = self.to_json().as_array() {
                entries = array.len();
                for entry in array {
                    match *entry {
                        serde_json::value::Value::Object(ref x) => {
                            for key in x.keys() {
                                let counter = out_hmap.entry(key.to_owned()).or_insert(0);
                                *counter += 1;
                            }
                        }
                        _ => continue,
                    }
                }
            } else {
                println!("no array");
            }

        (entries, out_hmap)
    }

    /// return the string in machinereadable format
    pub fn raw(&self) -> String {
        serde_json::to_string(&self.to_json()).unwrap_or_else(|_| "raw: problem with decoding".into())
    }

    /// return in human readable form
    pub fn human(&self) -> String {
        serde_json::to_string_pretty(&self.to_json()).unwrap_or_else(|_| "hum: problem with decoding".into())
    }

}

