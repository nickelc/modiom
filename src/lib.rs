#[macro_use]
extern crate serde_derive;

pub mod config;
pub mod manifest;
pub mod md5;
pub mod utils;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
pub type CliResult = std::result::Result<(), Box<dyn std::error::Error>>;
