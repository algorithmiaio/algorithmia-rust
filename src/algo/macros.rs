

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
    (text => Algo::$i:ident) => {
        algo_entrypoint!(&str, apply_str, Algo::$i);
    };
    (bytes => Algo::$i:ident) => {
        algo_entrypoint!(&[u8], apply_bytes, Algo::$i);
    };
    (JsonValue => Algo::$i:ident) => {
        algo_entrypoint!(&::algorithmia::algo::JsonValue, apply_json, Algo::$i);
    };
    (AlgoInput => Algo::$i:ident) => {
        algo_entrypoint!(::algorithmia::algo::AlgoInput, apply, Algo::$i);
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
    (text => $p:path) => {
        algo_entrypoint!(&str, apply_str, $p);
    };
    (bytes => $p:path) => {
        algo_entrypoint!(&[u8], apply_bytes, $p);
    };
    (JsonValue => $p:path) => {
        algo_entrypoint!(&::algorithmia::algo::JsonValue, apply_json, $p);
    };
    (AlgoInput => $p:path) => {
        algo_entrypoint!(::algorithmia::algo::AlgoInput, apply, $p);
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