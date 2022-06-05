# Pontus Onyx

A [remoteStorage](https://remotestorage.io/) server and client implemented in Rust.

```txt
⚠ Warning : it is not production-ready until version 1.0.0 !
So 0.x.0 versions are breaking changes until then.
```

Based on [IETF Draft of the 01 June 2022](https://datatracker.ietf.org/doc/html/draft-dejong-remotestorage-18).

This crate contains :

- a client library (to use with webassembly),
- a server library (for embeddable projects),
- a server binary (to use the server library directly).

## Development

### Run server

```cmd
cargo run --features server_bin
```
