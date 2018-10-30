
### 3.0

Breaking changes
- Simplify `AlgoInput` to remove references that weren't being used.
- Removed `algo::version` module. Use `user/algo/version` string instead.
- Moved entrypoint traits into new root module.
- Moved entrypoint functionality behind `entrypoint` feature flag.
- Replace `algo_entrypoint!` macro with `#[entrypoint]` attribute.
- Algorithmia client instantiation is now fallible instead of hiding errors until HTTP request
- `ApiAuth` type has been made private

TODO:
- Chaining entrypoint traits to accept `&mut self` to allow for easier state manipulation
- Async client
- Remove need to box entrypoint return types (either specialization or codegen the boxing)
- 2018 idioms
- Cleanup/fix/overhaul errors
- Combine AlgoInput and AlgoOutput
