Algorithmia Rust Client Library
-------------------------------

A rust client library to query the Algorithmia API.

## Usage

```rust
extern crate algorithmia;
use algorithmia::{Service, AlgorithmOutput};

// Initialize a Client with the API key
let client = Service::new("111112222233333444445555566");

// Specify the algorithm you want to execute
let mut factor = client.algorithm("kenny", "Factor");

// Run the algorithm using a type safe decoding of the output to Vec<int>
//   since this algorithm outputs results as a JSON array of integers
let output: AlgorithmOutput<Vec<int>> = try!(factor.query("19635".to_string()));
println!("Completed in {} seconds with result: {}", output.duration, output.result);

// Alternatively, query_raw will return the raw JSON string
let raw_output = try!(client.query_raw(algorithm, "19635"));
println!("Raw JSON output:\n{}", raw_output);
```

See [dijkstra.rs](examples/dijkstra.rs) for a more complete example using custom types for input and output.

## Build & Test

This project is built and tested with cargo:

    cargo build
    cargo test

## Examples

The examples directory (built with tests) contains additional samples.

### [algo](examples/algo.rs)

A sample CLI tool that uses `query_raw` to execute algorithms:

    $ export ALGORITHMIA_API_KEY=111112222233333444445555566
    $ target/examples/algo -d 19635 kenny/Factor
    {"duration":0.47086329,"result":[3,5,7,11,17]}

### [dijkstra](examples/dijkstra.rs)

A more complete type-safe example of using `query` to execute [kenny/Dijkstra](http://algorithmia.com/algorithms/kenny/Dijkstra).

    $ target/examples/dijkstra
    Input: [{"b":[2,2],"c":[3,3],"a":[1,1]},{"a":["b"],"b":["c"]},"a","c"]
    Shortest route: [a, b, c]
    Completed in 0.010614 seconds.

