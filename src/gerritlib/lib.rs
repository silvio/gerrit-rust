
//! library to `gerrit-rust` tool

#![feature(custom_derive)]
#![feature(plugin)]
#![feature(proc_macro)]

#[macro_use]
extern crate quick_error;

#[macro_use]
extern crate log;

extern crate curl;
extern crate git2;
extern crate regex;
extern crate url;
extern crate gron;

#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;

pub mod call;
pub mod changes;
pub mod entities;
pub mod error;
pub mod gerrit;

