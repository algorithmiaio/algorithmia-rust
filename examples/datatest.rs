extern crate algorithmia;
extern crate rustc_serialize;

use algorithmia::Service;
// use algorithmia::collection::*;
use std::env;
// use rustc_serialize::{json};



fn main() {
    let mut args = env::args();
    args.next(); // discard args[0]
    let path = args.next().unwrap();

    let api_key = match env::var("ALGORITHMIA_API_KEY") {
        Ok(key) => key,
        Err(e) => { panic!("ERROR: unable to get ALGORITHMIA_API_KEY: {}", e); }
    };

    let service = Service::new(&*api_key);
    match service.clone().collection(&*path).create() {
        Ok(_) => println!("Successfully created collection {}", path),
        Err(e) => println!("ERROR creating collection: {:?}", e),
    }

    match service.clone().collection(&*path).delete() {
        Ok(_) => println!("Successfully deleted collection {}", path),
        Err(e) => println!("ERROR deleting collection: {:?}", e),
    }


}
