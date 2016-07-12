Algorithmia Rust Client Library
-------------------------------

A rust client library for the Algorithmia API.

[Documentation](http://algorithmiaio.github.io/algorithmia-rust/algorithmia/)

[![Build Status](https://travis-ci.org/algorithmiaio/algorithmia-rust.svg)](https://travis-ci.org/algorithmiaio/algorithmia-rust)
[![Crates.io](https://img.shields.io/crates/v/algorithmia.svg?maxAge=2592000)](https://crates.io/crates/algorithmia)

## Getting started

The Algorithmia Rust client is published to [crates.io](https://crates.io/crates/algorithmia).
Add `algorithmia = "1.3.0"` to the dependencies in your `Cargo.toml` and run `cargo install`.

Instantiate an Algorithmia client using your API key:

```rust
use algorithmia::*;

let client = Algorithmia::client("YOUR_API_KEY");
```

Now you're ready to call algorithms.

## Calling algorithms

This client provides an `Algorithm` type (generally created by `client.algo(..)`) which provides
methods for calling an algorithm hosted on the Algorithmia platform.
The following examples of calling algorithms are organized by type of input/output which vary between algorithms.

Note: a single algorithm may have different input and output types, or accept multiple types of input,
so consult the algorithm's description for usage examples specific to that algorithm.

### Text input/output

Call an algorithm with text input by simply passing `&str` into `Algorithm::pipe`.
If the algorithm output is text, call the `as_string` method on the response.

```rust
let algo = client.algo("algo://demo/Hello/0.1.1");
let response = algo.pipe("HAL 9000").unwrap();
println!("{}", response.as_string().unwrap());
```

### JSON input/output

Call an algorithm with JSON input calling `pipe` with a reference to a type that implements `rustc-serialize::Encodable`.
If the algorithm output is JSON, you can call `decode` to deserialize the resonse into a type that implements `rustc-serialize::Decodable`.

This includes many primitive types, tuples, `Vec`, and other collection types from the standard library:

```rust
let algo = client.algo("algo://WebPredict/ListAnagrams/0.1.0");
let response = algo.pipe(vec!["transformer", "terraforms", "retransform"]).unwrap();
let output: Vec<String> = response.decode().unwrap();
// -> ["transformer", "retransform"] as Vec<String>
```

Implementing `Encodable` or `Decodable` for your custom types is generally as easy as adding a `derive` annotation.

```rust
#[derive(RustcDecodable, RustcEncodable)]
struct MyStruct {
    some_field: String,
    other_field: u32,
}
// now you can call `pipe` with `&MyStruct` or `decode` into `MyStruct`
```

Alternatively, you can work with raw JSON:

```rust
let response = algo.pipe_json(r#"["transformer", "terraforms", "retransform"]"#);
let output = response.as_json().unwrap().to_string();
// -> "[\"transformer\", \"retransform\"]"
```

[Open an issue](https://github.com/algorithmiaio/algorithmia-rust/issues) if you really want `serde-json` support. :-)

### Binary input/output

Call an algorithm with binary input by calling the `pipe` method with a slice of bytes (`&[u8]`).
If the algorithm response is binary data, then call the `as_bytes` method on the response
to obtain a byte vector (`Vec<u8>`).

```rust
let mut input = Vec::new();
File::open("/path/to/bender.jpg").read_to_end(&mut input);
let response = client.algo("opencv/SmartThumbnail/0.1").pipe(&input).unwrap();
let output = response.as_bytes().unwrap();
// -> Vec<u8>
```

### Error handling

True to the nature of explicit error handling in rust,
the `pipe` and response parsing methods all return `Result`-wrapped types
intended to be handled with `match` blocks or the `try!` macro:

```rust
let algo = client.algo("algo://demo/Hello/0.1.1");
match algo.pipe(&[]) {
    Ok(response) => { /* success */ },
    Err(err) => println!("error calling demo/Hello: {}", err),
}
// -> error calling demo/Hello: apply() functions do not match input data
```

### Request options

The client exposes options that can configure algorithm requests via a builder pattern.
This includes support for changing the timeout or indicating that the API should include stdout in the response.

```rust
let mut algo = client.algo("algo://demo/Hello/0.1.1");
let algo = algo.timeout(10).enable_stdout();
let response = algo.pipe(input).unwrap();
if let Some(ref stdout) = response.metadata.stdout {
    println!("{}", stdout);
}
```

Note: `enable_stdout()` is ignored if you do not have access to the algorithm source.


## Examples

For examples of using this client, see:

- Basic test [examples](https://github.com/algorithmiaio/algorithmia-rust/tree/master/examples)
- [Algorithmia CLI](https://github.com/algorithmiaio/algorithmia-cli) built with this client
- [Algorithmia FUSE](https://github.com/anowell/algorithmia-fuse) built with this client

## Build & Test

This project is built and tested with cargo:

```bash
cargo build
cargo test
cargo doc --no-deps
```

Pro-tip: before building docs, clone existing docs to track changes
```bash
git clone -b gh-pages git@github.com:algorithmiaio/algorithmia-rust.git target/doc
```

