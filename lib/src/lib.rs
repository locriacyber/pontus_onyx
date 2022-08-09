#![allow(clippy::needless_return)]
#![allow(non_snake_case)]

pub mod scope;

pub mod item;

#[cfg(feature = "client")]
pub mod client;

#[cfg(feature = "server")]
pub mod database;

#[cfg(feature = "assets")]
pub mod assets;

#[cfg(feature = "actix_server")]
pub mod http_server;
