
//! library to `gerrit-rust` tool

#![feature(custom_derive)]
#![feature(plugin)]
#![feature(proc_macro)]

#[macro_use] extern crate log;
#[macro_use] extern crate quick_error;
#[macro_use] extern crate serde_derive;
extern crate curl;
extern crate git2;
extern crate semver;
extern crate serde;
extern crate serde_json;
extern crate url;

pub mod call;
pub mod changes;
pub mod config;
pub mod entities;
pub mod error;
pub mod gerrit;
