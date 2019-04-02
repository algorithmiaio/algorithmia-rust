use algorithmia::prelude::*;
use std::error::Error;

fn apply(input: u32) -> Result<u32, Box<Error>> {
    Ok(input + 42)
}

fn main() -> Result<(), Box<Error>> {
    setup_handler(apply)
}
