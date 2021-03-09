//! Models, APIs and tools for OGC technologies.
//!
//! This crate is dedicated to creating idiomatic Rust APIs for interacting
//! with OGC services.
//!
//! This version supports:
//!  * WMS GetCapabilities
//!  * WMS GetMap
//!
//! The planned order of implementation is
//!  1. WMS
//!  2. WFS
//!  3. Core API
#![allow(dead_code)]
#[macro_use]
extern crate serde_derive;
extern crate anyhow;
extern crate serde;
extern crate serde_json;
extern crate serde_xml_rs;

pub mod wms;
pub mod parser;