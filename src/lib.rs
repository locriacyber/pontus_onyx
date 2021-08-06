#![allow(clippy::needless_return)]
#![allow(non_snake_case)]

/*! pontus_onyx is a [remoteStorage](https://remotestorage.io/) server and client implemented in Rust.

Features :

- `server_bin`
- `server_lib`
  - `server_file_storage`
  - `server_local_storage`
- `client_lib`
*/

mod scope;
pub use scope::*;

pub mod item;

#[cfg(feature = "client_lib")]
pub mod client;

#[cfg(feature = "server_lib")]
pub mod database;
