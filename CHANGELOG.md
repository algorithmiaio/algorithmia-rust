### 3.0

Major version to reduce complexity and make the client easier to use.

**Breaking changes**
- Upgrade `serde` to 1.0 and completely remove `rustc_serialize`
- Combined `AlgoInput` and `AlgoOutput` into a simpler `AlgoIo` that removes references that weren't being used.
- Removed `algo::version` module. Use `user/algo/version` string instead.
- Moved entrypoint traits into new root module.
- Moved entrypoint functionality behind `entrypoint` feature flag.
- Replace `algo_entrypoint!` macro with `#[entrypoint]` attribute.
- Algorithmia client instantiation is now fallible instead of hiding errors until HTTP request
- `ApiAuth` type has been made private
- Entrypoint traits accept `&mut self` to allow for easier state manipulation
- Entrypoint codegen autoboxes return types (for lack of specialization)
- Error API surface decreased significantly

# TODO
- Make AlgoIo opaque - not exposing serde types
- Experiment with reqwest::async
