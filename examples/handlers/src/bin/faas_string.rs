use algorithmia::prelude::*;

fn apply(input: String) -> Result<String, String> {
    Ok(format!("Hello {}", input))
}

fn main() {
    handler::run(apply)
}
