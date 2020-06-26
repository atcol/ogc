#![allow(dead_code)]
/// APIs for OGC services.
#[macro_use]
extern crate serde_derive;
extern crate anyhow;
extern crate serde;
extern crate serde_json;
extern crate serde_xml_rs;
extern crate tokio;
pub mod wms;
