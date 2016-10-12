#![cfg_attr(feature = "serde_derive", feature(proc_macro))]

#[macro_use]
extern crate enum_primitive;
extern crate serde;
extern crate serde_json;

#[cfg(feature = "serde_derive")]
#[macro_use]
extern crate serde_derive;

#[cfg(feature = "serde_derive")]
include!("lib.rs.in");

#[cfg(feature = "serde_codegen")]
include!(concat!(env!("OUT_DIR"), "/lib.rs"));
