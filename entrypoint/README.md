# Generate entrypoint for a rust algorithm

Annotate a free function or the impl block of a type with `#[entrypoint]` to generate the required `EntryPoint`/`DecodedEntryPoint` implementations.

## Getting started

To set a simple `apply` function as the algorithm entrypoint, just annotate the method with `#[entrypoint]`:

```rust
#![feature(proc_macro)]
extern crate algorithmia;
use algorithmia::prelude::*;

#[entrypoint]
fn apply(name: String) -> Result<String, String> {
    unimplemented!()
}
```

Note: `feature(proc_macro)` and a nightly compiler are necessary until `proc_macro_attribute` stabilizes (likely many months away still).

## Automatic JSON serialization/deserialization

To use your own custom types as input and output, simply implement `Deserialize` and `Serialize` respectively.

```rust
#[derive(Deserialize)
struct Input { titles: Vec<String>, max: u32 }

#[derive(Serialize)
struct Output { titles: Vec<String> }

#[entrypoint]
fn start(input: Input) -> Result<Output, String> {
    unimplemented!();
}
```

Note: this currently depends on specialization to serialize unboxed output.

## Preloading (Advanced Usage)

If your algorithm has a preload step that doesn't vary with user input (e.g. loading a model), you can create a type that implements `Default`, and use a method on that type as your entrypoint (the `#[entrypoint]` annotation goes on the impl of your type. Multiple API calls in succession from a single user will only instantiate the type once, but call `apply` multiple times:

```rust
#[derive(Deserialize)
struct Input { titles: Vec<String>, max: u32 }

#[derive(Serialize)
struct Output { titles: Vec<String> }

struct App { model: Vec<u8> }

#[entrypoint]
impl App {
    fn apply(input: Input) -> Result<Output, String> {
        unimplemented!();
    }
}

impl Default for App {
    fn default() -> Self {
        App { model: load_model() }
    }
}
```

## Input/Output types:

**Valid input**:
- String types (e.g. `&str` and `String`
- Byte types (e.g. `&[u8]` and `Vec<u8>`
- Json `&Value` type
- AlgoIo enum type (for matching on text, json, and binary input)
- Any type that implements `serde::Deserialize` (e.g. `#[derive(Deserialize)]`

**Valid output (`Ok` variant of return value)**:
- `String`, `Vec<u8>`, `Value`, `AlgoIo`
- Any type that implements `serde::Serialize` (e.g. `#[derive(Serialize)]`

**Valid error types (`Err` variant of return value)**:
Anything with an conversion to `Box<Error>`. This includes `String` and basically any type that implements the `Error` trait.

## Future work:

- Allow specifying `apply` and `default` methods in advanced cases
- Better errors
  - Can't attach entrypoint to `struct Algo`
  - Assertions and guidance on valid return types
- Add tests
