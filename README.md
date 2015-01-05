Algorithmia Rust Client Library
-------------------------------

A rust client library to query the Algorithmia API.

## Usage

```rust
extern crate algorithmia;
use algorithmia::{ Algorithm, Client, Output };

// Initialize a Client with the API key
let client = Client::new("111112222233333444445555566");

// Specify the algorithm you want to execute
let factor = Algorithm::new("kenny", "Factor");

// Run the algorithm with input data
//   using a type safe decoding of the output
// The "kenny/Factor" algorithm outputs
//   results as a JSON array of integers
//   which decodes into Vec<int>
let output: Output<Vec<int>> = try!(client.query(factor, "19635"));
println!("Completed in {} seconds with result: {}", output.duration, output.result);

// Alternatively, query_raw will return the raw JSON string
let raw_output = try!(client.query_raw(algorithm, "19635"));
println!("Raw JSON output:\n{}", raw_output);
```

## Build & Test

This project is built and tested with cargo:

    cargo build
    cargo test

## Examples

The examples directory (built with tests) also contains a sample CLI tool that uses this library to execute algorithms:

    $ export ALGORITHMIA_API_KEY=111112222233333444445555566
    $ target/examples/algo kenny/Factor 19635
    {"duration":0.47086329,"result":[3,5,7,11,17]}


