use algorithmia::prelude::*;
use std::error::Error;

fn apply(input: AlgoIo) -> Result<AlgoIo, Box<Error>> {
    Ok(input)
}

fn main() {
    setup_handler(apply)
}
