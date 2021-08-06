# Pontus Onyx

A [remoteStorage](https://remotestorage.io/) server and client implemented in Rust.

```txt
⚠ Warning : it is not production-ready until version 1.0.0 !
So 0.x.0 versions are breaking changes until then.
```

Based on [IETF Draft of the 14 June 2021](https://datatracker.ietf.org/doc/html/draft-dejong-remotestorage-17).

This crate should contain client library, server library (for embeddable projects), and server binary.

## Development

### Run server

```cmd
cargo run --features server_bin
```
