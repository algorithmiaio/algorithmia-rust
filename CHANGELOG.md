
### 3.0

Breaking changes
- Combined `AlgoInput` and `AlgoOutput` into a simpler `AlgoIo` that removes references that weren't being used.
- Removed `algo::version` module. Use `user/algo/version` string instead.
- Moved entrypoint traits into new root module.
- Moved entrypoint functionality behind `entrypoint` feature flag.
- Replace `algo_entrypoint!` macro with `#[entrypoint]` attribute.
- Algorithmia client instantiation is now fallible instead of hiding errors until HTTP request
- `ApiAuth` type has been made private
- Entrypoint traits accept `&mut self` to allow for easier state manipulation

TODO:
- Async client
- Remove need to box entrypoint return types (either specialization or codegen the boxing)
