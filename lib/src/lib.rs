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
