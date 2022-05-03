#![allow(clippy::needless_return)]
#![allow(non_snake_case)]

/*! pontus_onyx is a [remoteStorage](https://remotestorage.io/) server and client implemented in Rust.

Cargo features :

- `server_bin`
- `server_lib`
  - `server_file_storage`
  - `server_local_storage`
- `client_lib`
  - `client_lib_cookies`
*/

pub mod scope;

pub mod item;

#[cfg(feature = "client_lib")]
pub mod client;

#[cfg(feature = "server_lib")]
pub mod database;
