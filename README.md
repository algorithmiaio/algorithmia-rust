Algorithmia Rust Client Library
-------------------------------

A rust client library for the Algorithmia API.

[![Build Status](https://travis-ci.org/anowell/algorithmia_rust.svg)](https://travis-ci.org/anowell/algorithmia_rust)

## Library Usage

```rust
extern crate algorithmia;
use algorithmia::Service;
use algorithmia::algorithm::AlgorithmOutput;

// Initialize with an API key
let algo_service = Service::new("111112222233333444445555566");
let mut factor = algo_service.algorithm("kenny", "Factor");

// Run the algorithm using a type safe decoding of the output to Vec<int>
//   since this algorithm outputs results as a JSON array of integers
let input = "19635".to_string();
let output: AlgorithmOutput<Vec<i64>> = factor.exec(&input).unwrap();
println!("Completed in {} seconds with result: {:?}", output.duration, output.result);

// Alternatively, exec_raw will return the raw JSON string
let raw_output = try!(algo_service.exec_raw(algorithm, "19635"));
println!("Raw JSON output:\n{}", raw_output);

// Work with data collections
let my_bucket = algo_service.collection("my_user", "my_bucket");
my_bucket.create();
let mut my_file = File::open("/path/to/file").unwrap();
my_bucket.upload_file(my_file);
my_bucket.write_file("some_filename", "file_contents".as_bytes());
```

See [dijkstra.rs](examples/dijkstra.rs) for a more complete example using custom types for input and output.


## CLI Usage

### [algo](src/bin/algo.rs)

A sample CLI tool that uses `exec_raw` to execute algorithms:

    $ export ALGORITHMIA_API_KEY=111112222233333444445555566
    $ target/examples/algo -d 19635 kenny/Factor
    {"duration":0.47086329,"result":[3,5,7,11,17]}

### [algodata](src/bin/algodata.rs)

A sample CLI tool to interact with the Algorithmia Data API

    $ algodata anowell/foo create
    CollectionCreated { collection_id: 180, object_id: "01234567-abcd-1234-9876-111111111111", collection_name: "foo", username: "anowell", acl: CollectionAcl { read_w: false, read_g: false, read_u: true, read_a: true } }

    $ algodata anowell/foo upload *.png
    Uploading /home/anowell/Pictures/collections.png
    Uploading /home/anowell/Pictures/notif-basic.png
    Uploading /home/anowell/Pictures/publish_menu.png
    Finished uploading 3 file(s)

    $ algodata anowell/foo
    CollectionShow { username: "anowell", collection_name: "foo3", files: ["collections.png", "notif-basic.png", "publish_menu.png"] }


## Build & Test

This project is built and tested with cargo:

    cargo build
    cargo test

