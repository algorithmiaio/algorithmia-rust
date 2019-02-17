Algorithmia Rust Library
-------------------------------

A rust client library for the Algorithmia API is split into 2 parts:.

[Algorithmia Rust Client](algorithmia/README.md) - Used to call algorithms and manage data via the Algorithmia platform
[Entrypoint Generator](entrypoint/README.md) - Procedural macro to generate entrypoint boilerplate when authoring Rust algorithms on the Algorithmia platform. Re-exported by the Algorithmia Rust Client as `algorithmia::entrypoint::entrypoint`

[Documentation](http://algorithmiaio.github.io/algorithmia-rust/algorithmia/)

[![Build Status](https://travis-ci.org/algorithmiaio/algorithmia-rust.svg)](https://travis-ci.org/algorithmiaio/algorithmia-rust)
[![Crates.io](https://img.shields.io/crates/v/algorithmia.svg?maxAge=2592000)](https://crates.io/crates/algorithmia)

## Build & Test

This project is built and tested with cargo:

```bash
cargo build
cargo test
cargo doc --no-deps
```
