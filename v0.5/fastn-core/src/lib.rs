#![allow(clippy::derive_partial_eq_without_eq, clippy::get_first)]
#![deny(unused_crate_dependencies)]
#![warn(clippy::used_underscore_binding)]

extern crate self as fastn_core;

mod config;
mod route;

pub use config::{Config, DynamicRoute, Redirect, Sitemap};
pub use route::Route;

// this type is not really needed here, but adding here because fastn-section is our lowest
// level crate
#[derive(Debug, Clone)]
pub struct AutoImport {}
