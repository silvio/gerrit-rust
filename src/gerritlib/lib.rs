
//! library to `gerrit-rust` tool

#![feature(custom_derive)]
#![feature(plugin)]
#![feature(custom_attribute)]
#![feature(proc_macro)]

#[macro_use]
extern crate quick_error;

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

