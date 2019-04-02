use algorithmia::prelude::*;

fn apply(input: String) -> Result<String, String> {
    Ok(format!("Hello {}", input))
}

fn main() {
    setup_handler(apply)
}
