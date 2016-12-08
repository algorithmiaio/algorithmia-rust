/// Macro for implementing `Entrypoint` or `DecodedEntryPoint` boilerplate in an algorithm
///
/// Algorithmia support for Rust algorithms is hard-coded to call `Algo::apply(AlgoInput)`
///   but for the sake of convenience the `EntryPoint` trait provides a default implementation
///   of `apply` that delegates to `apply_str`, `apply_bytes`, or `apply_json` depending on
///   the variant of `AlgoInput`. The `DecodedEntryPoint` trait overrides the default `apply`
///   to provide a method that uses a deserialized associated type when calling `apply_decoded`.
///   This pattern is incredibly flexible, but adds boilerplate and still requires
///   converting output and error types back.
///
/// The `algo_entrypoint!` macro hides all that boilerplate code behind a single macro invocation.
///   `algo_entrypoint(type => your_fn)` wires up the boilerplate for calling `your_fn(type)`:
///
/// Use the following types:
///
/// - `&str` if your algorithm accepts text input
/// - `&[u8]` if your algorithm accepts binary input
/// - Any deserializeable type if your algorithm accepts JSON input
/// - `&JsonValue` is you want your algorithm to work directly with the typed JSON input
/// - `AlgoInput` if you want to work with the full enum of possible input types
///
/// In all cases, the return value of `your_fn` should be `Result<T, E>` where:
///
/// - `T` impl `Into<AlgoOutput>` which includes `String`, `Vec<u8>`, `JsonValue`, and boxed* serializeable types
/// - `E` impl `Into<Box<Error>>` which includes `String` and all `Error` types
///
/// *&ast; support for unboxed serializeable depends on specialization implemented behind the 'nightly' feature.*
///
/// # Examples
///
/// ## Text Input/Output
/// To set the entrypoint to a function that receives and returns text:
///
/// ```no_run
/// # #[macro_use] extern crate algorithmia;
/// # fn main() {}
/// # use algorithmia::prelude::*;
/// algo_entrypoint!(&str => hello_text);
///
/// fn hello_text(input: &str) -> Result<String, String> {
///     unimplemented!()
/// }
/// ```
///
/// which generates:
///
/// ```ignore
/// #[derive(Default)]
/// pub struct Algo;
///
/// impl EntryPoint for Algo {
///     fn apply_str(&self, input: &str) -> Result<AlgoOutput, Box<::std::error::Error>> {
///         hello_text(input).map(AlgoOutput::from).map_err(|err| err.into())
///     }
/// }
///
/// fn hello_text(input: &str) -> Result<String, String> {
///     unimplemented!()
/// }
/// ```
///
/// ## Binary Input/Output
/// To set the entrypoint to a function that receives and returns binary data:
///
/// ```no_run
/// # #[macro_use] extern crate algorithmia;
/// # fn main() {}
/// # use algorithmia::prelude::*;
/// algo_entrypoint!(&[u8] => hello_bytes);
///
/// fn hello_bytes(input: &[u8]) -> Result<Vec<u8>, String> {
///     unimplemented!()
/// }
/// ```
///
/// ## JSON Input/Output
/// To set the entrypoint to a function that receives and returns JSON data,
///   you can work directly with the `JsonValue`:
///
/// ```no_run
/// # #[macro_use] extern crate algorithmia;
/// # fn main() {}
/// # use algorithmia::prelude::*;
/// algo_entrypoint!(&JsonValue => hello_json);
///
/// fn hello_json(input: &JsonValue) -> Result<JsonValue, String> {
///     unimplemented!()
/// }
/// ```
///
/// Although, often the preferred way to work with JSON is to automatically
///   deserialize into and serialize from a custom type.
///
/// ```ignore
/// # #[macro_use] extern crate algorithmia;
/// # fn main() {}
/// # use algorithmia::prelude::*;
/// #[derive(RustcDecodable)]
/// pub struct MyInput { corpus: String, msg: String }
///
/// #[derive(RustcEncodable)]
/// pub struct MyOutput { probabilities: Vec<(String, f32)> }
///
/// algo_entrypoint!(MyInput => hello_custom);
///
/// fn hello_custom(input: MyInput) -> Result<Box<MyOutput>, String> {
///     unimplemented!()
/// }
/// ```
///
/// Note: `Box` for serializeable output is currently required for lack of specialization.
///   The specialization implementation already exists behind the `nightly` feature.
///   Alternatively, you can return `AlgoOutput` via `AlgoOutput::from(&myOutput)`
///
/// The previous example expands into:
///
/// ```ignore
/// #[derive(RustcDecodable)]
/// pub struct MyInput { corpus: String, msg: String };
///
/// #[derive(RustcEncodable)]
/// pub struct MyOutput { probabilities: Vec<(String, f32)> };
///
/// #[derive(Default)]
/// pub struct Algo;
///
/// impl DecodedEntryPoint for Algo {
///     type Input (String, String);
///     fn apply_decoded(&self, input: MyInput) -> Result<AlgoOutput, Box<::std::error::Error>> {
///         hello_custom(input).map(AlgoOutput::from).map_err(|err| err.into())
///     }
/// }
///
/// fn hello_custom(input: MyInput) -> Result<Box<MyOutput>, String> {
///     unimplemented!()
/// }
/// ```
///
/// ## Customizing `Default`
/// And finally, it is possible to provide a custom `Default` implementation
///   and pass that state into your method by prefixing `your_fn` with
///   the `Algo::` namespace.
///
/// ```no_run
/// # #[macro_use] extern crate algorithmia;
/// # fn main() {}
/// # use algorithmia::prelude::*;
/// pub struct Algo{ init_id: String }
///
/// algo_entrypoint!(&str => Algo::hello_text);
///
/// impl Algo {
///     fn hello_text(&self, input: &str) -> Result<String, String> {
///         unimplemented!()
///     }
/// }
///
/// impl Default for Algo {
///     fn default() -> Algo {
///         Algo { init_id: "foo".into() }
///     }
/// }
/// ```
#[macro_export]
macro_rules! algo_entrypoint {
    // Helpers for recursively implementing text/bytes/JsonValue/AlgoInput
    ($t:ty, $apply:ident, Algo::$method:ident) => {
        impl EntryPoint for Algo {
            fn $apply(&self, input: $t) -> Result<AlgoOutput, Box<::std::error::Error>> {
                self.$method(input).map(AlgoOutput::from).map_err(|err| err.into())
            }
        }
    };
    ($t:ty, $apply:ident, $p:path) => {
        #[derive(Default)] pub struct Algo;
        impl EntryPoint for Algo {
            fn $apply(&self, input: $t) -> Result<AlgoOutput, Box<::std::error::Error>> {
                $p(input).map(AlgoOutput::from).map_err(|err| err.into())
            }
        }
    };

    // Implement EntryPoint to call methods on `Algo`
    (&str => Algo::$i:ident) => {
        algo_entrypoint!(&str, apply_str, Algo::$i);
    };
    (&[u8] => Algo::$i:ident) => {
        algo_entrypoint!(&[u8], apply_bytes, Algo::$i);
    };
    (&JsonValue => Algo::$i:ident) => {
        algo_entrypoint!(&JsonValue, apply_json, Algo::$i);
    };
    (AlgoInput => Algo::$i:ident) => {
        algo_entrypoint!(AlgoInput, apply, Algo::$i);
    };
    ($t:ty => Algo::$i:ident) => {
        impl DecodedEntryPoint for Algo {
            type Input = $t;
            fn apply_bytes(&self, input: $t) -> Result<AlgoOutput, Box<::std::error::Error>> {
                self.$i(input).map(AlgoOutput::from).map_err(|err| err.into())
            }
        }
    };

    // Implement EntryPoint to call free functions
    (&str => $p:path) => {
        algo_entrypoint!(&str, apply_str, $p);
    };
    (&[u8] => $p:path) => {
        algo_entrypoint!(&[u8], apply_bytes, $p);
    };
    (&JsonValue => $p:path) => {
        algo_entrypoint!(&JsonValue, apply_json, $p);
    };
    (AlgoInput => $p:path) => {
        algo_entrypoint!(AlgoInput, apply, $p);
    };

    ($t:ty => $p:path) => {
        #[derive(Default)] pub struct Algo;
        impl DecodedEntryPoint for Algo {
            type Input = $t;
            fn apply_decoded(&self, input: $t) -> Result<AlgoOutput, Box<::std::error::Error>> {
                $p(input).map(AlgoOutput::from).map_err(|err| err.into())
            }
        }
    };
}

// Testing all the variants with unused code
mod test_str {
    use prelude::*;
    algo_entrypoint!(&str => hello_text);
    fn hello_text(_input: &str) -> Result<String, String> { unimplemented!() }
}

mod test_bytes {
    use prelude::*;
    algo_entrypoint!(&[u8] => hello_bytes);
    fn hello_bytes(_input: &[u8]) -> Result<Vec<u8>, String> { unimplemented!() }
}

mod test_json {
    use prelude::*;
    algo_entrypoint!(&JsonValue => hello_json);
    fn hello_json(_input: &JsonValue) -> Result<JsonValue, String> { unimplemented!() }
}

mod test_enum {
    use prelude::*;
    algo_entrypoint!(AlgoInput => hello_enum);
    fn hello_enum(_input: AlgoInput) -> Result<AlgoOutput, String> { unimplemented!() }
}

mod test_decode {
    use prelude::*;

    #[cfg_attr(feature="with-rustc-serialize", derive(RustcDecodable, RustcEncodable))]
    #[cfg_attr(feature="with-serde", derive(Deserialize, Serialize))]
    pub struct Custom {
        foo: String,
        bar: Vec<u32>,
        baz: bool,
    }
    algo_entrypoint!(Custom => hello_decoded);
    fn hello_decoded(_input: Custom) -> Result<Box<Custom>, String> { unimplemented!() }
}