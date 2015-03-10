Algorithmia Rust Client Library
-------------------------------

A rust client library to query the Algorithmia API.

## Usage

```rust
extern crate algorithmia;
use algorithmia::{Service, AlgorithmOutput};

// Initialize a Client with the API key
let client = Service::new("111112222233333444445555566");
let mut factor = client.algorithm("kenny", "Factor");

// Run the algorithm using a type safe decoding of the output to Vec<int>
//   since this algorithm outputs results as a JSON array of integers
let output: AlgorithmOutput<Vec<int>> = try!(factor.query("19635".to_string()));
println!("Completed in {} seconds with result: {}", output.duration, output.result);

// Alternatively, query_raw will return the raw JSON string
let raw_output = try!(client.query_raw(algorithm, "19635"));
println!("Raw JSON output:\n{}", raw_output);

// Create data collections for algorithms to use
let my_bucket = service.collection("my_user", "my_bucket");

// Create or upload files to your data collection
// Coming soon...
// my_bucket.upload_file(...)
// my_bucket.write_file(...)
```

See [dijkstra.rs](examples/dijkstra.rs) for a more complete example using custom types for input and output.

## Build & Test

This project is built and tested with cargo:

    cargo build
    cargo test

## Tools & Examples

The [src/bin](src/bin) and [examples](examples) directories contain additional samples.

### [algo](src/bin/algo.rs)

A sample CLI tool that uses `query_raw` to execute algorithms:

    $ export ALGORITHMIA_API_KEY=111112222233333444445555566
    $ target/examples/algo -d 19635 kenny/Factor
    {"duration":0.47086329,"result":[3,5,7,11,17]}

### [algodata](src/bin/algodata.rs)

A sample CLI tool to interact with the Algorithmia Data API

    $ algodata anowell/rustfoo create
    CollectionCreateResponse { stream_id: 123, object_id: "01234567-abcd-1234-9876-111111111111", stream_name: "rustfoo", username: "anowell", acl: "6004" }

    $ algodata anowell/rustfoo upload *.png
    Uploading /home/anowell/Pictures/collections.png
    Uploading /home/anowell/Pictures/notif-basic.png
    Uploading /home/anowell/Pictures/publish_menu.png
    Finished uploading 3 file(s)

### [dijkstra](examples/dijkstra.rs)

A more complete type-safe example of using `query` to execute [anowell/Dijkstra](http://algorithmia.com/algorithms/anowell/Dijkstra).

    $ target/examples/dijkstra
    Input:
    [ {
        "b": {"c": 2, "a": 2 },
        "c": {"b": 2, "d": 1 },
        "d": {"c": 3, "a": 1 },
        "a": {"b": 1 }
      },
      "a",
      "c"
    ]
    Shortest route: ["a", "b", "c"]
    Completed in 0.001022 seconds.
