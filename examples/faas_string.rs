use algorithmia::prelude::*;
use std::error::Error;

fn apply(input: String) -> Result<String, String> {
    Ok(format!("Hello {}", input))
}

fn main() -> Result<(), Box<Error>> {
    setup_handler(apply)
}
