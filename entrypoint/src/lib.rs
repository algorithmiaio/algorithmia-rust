#![recursion_limit = "1024"]

extern crate proc_macro;
extern crate syn;
#[macro_use]
extern crate quote;

use crate::proc_macro::TokenStream;
use quote::{ToTokens, Tokens};
use syn::*;

#[proc_macro_attribute]
pub fn entrypoint(_args: TokenStream, input: TokenStream) -> TokenStream {
    // TODO: support #[entrypoint(apply=run, default=init)] when attached to `impl`
    // let args_path = parse_path(&args.to_string()).ok();
    let item =
        parse_item(&input.to_string()).expect("#[entrypoint] must be attached to a function");

    let entrypoint = Entrypoint::from_item(item);
    entrypoint
        .output()
        .parse()
        .expect("#[entrypoint] failed to parse output")
}

struct Entrypoint {
    fn_name: Ident,
    self_type: Option<Ty>,
    input_type: Ty,
    item: Item,
}

impl Entrypoint {
    fn from_item(item: Item) -> Self {
        let item_clone = item.clone();
        match item.node {
            ItemKind::Fn(fn_decl, unsafety, _constness, _abi, generics, _block) => {
                assert_eq!(
                    unsafety,
                    Unsafety::Normal,
                    "Unsafe #[entrypoint] functions are not supported"
                );
                assert_generics_empty(&generics);

                assert_eq!(
                    fn_decl.inputs.len(),
                    1,
                    "#[entrypoint] function may only take one arg"
                );
                let input_type = match fn_decl.inputs[0] {
                    FnArg::SelfRef(..) | FnArg::SelfValue(..) => {
                        panic!("Attach #[entrypoint] to impl block to support &self arg")
                    }
                    FnArg::Captured(_, ref ty) => ty.clone(),
                    FnArg::Ignored(ref ty) => ty.clone(),
                };

                // TODO: assert that fn_decl.output includes a `Result` token

                Entrypoint {
                    fn_name: item.ident,
                    self_type: None,
                    input_type: input_type,
                    item: item_clone,
                }
            }

            ItemKind::Impl(unsafety, _polarity, generics, _path, ty, impl_items) => {
                assert_eq!(
                    unsafety,
                    Unsafety::Normal,
                    "Unsafe #[entrypoint] impls are not supported"
                );
                assert_generics_empty(&generics);
                // TODO: assert that `ty` isn't of type `Algo` which would cause name collisions

                let method_name = Ident::new("apply");

                let fn_decl = impl_items
                    .iter()
                    .filter_map(|item| match item.node {
                        ImplItemKind::Method(ref sig, _) => Some((&item.ident, &sig.decl)),
                        _ => None,
                    })
                    .find(|pair| pair.0 == &method_name)
                    .map(|pair| pair.1.clone())
                    .expect("#[entrypoint] impl must include 'apply' method");

                assert_eq!(
                    fn_decl.inputs.len(),
                    2,
                    "#[entrypoint] within an impl must take &self and an input arg"
                );
                let input_type = match fn_decl.inputs[1] {
                    FnArg::SelfRef(..) | FnArg::SelfValue(..) => {
                        panic!("Are you using self as a second argument?")
                    }
                    FnArg::Captured(_, ref ty) => ty.clone(),
                    FnArg::Ignored(ref ty) => ty.clone(),
                };

                // TODO: assert that fn_decl.output includes a `Result` token

                Entrypoint {
                    fn_name: method_name,
                    self_type: Some(ty.as_ref().clone()),
                    input_type: input_type,
                    item: item_clone,
                }
            }

            _ => panic!("#[entrypoint]` attribute must be attached to a function or impl"),
        }
    }

    fn output(&self) -> Tokens {
        let mut input_type_tokens = Tokens::new();
        self.input_type.to_tokens(&mut input_type_tokens);

        match input_type_tokens.as_str() {
            "& str" | "String" => self.impl_entrypoint(Ident::new("apply_str"), "&str"),
            "& [ u8 ]" | "Vec < u8 >" => self.impl_entrypoint(Ident::new("apply_bytes"), "&[u8]"),
            "& Value" => self.impl_entrypoint(Ident::new("apply_json"), "&Value"),
            "AlgoInput" => self.impl_entrypoint(Ident::new("apply"), "AlgoInput"),
            _ => self.impl_decoded_entrypoint(),
        }
    }

    fn impl_entrypoint(&self, apply_fn: Ident, input_type: &str) -> Tokens {
        let ref item = self.item;
        let ref fn_name = self.fn_name;
        let input_type = parse_type(input_type).unwrap();

        // TODO: if specialization hasn't landed, consider generating the auto-boxing code for Serialize types
        match self.self_type {
            Some(ref self_type) => {
                quote! {
                    pub struct Algo(#self_type);
                    impl algorithmia::entrypoint::EntryPoint for Algo {
                        fn #apply_fn(&mut self, input: #input_type) -> ::std::result::Result<algorithmia::algo::AlgoIo, Box<::std::error::Error>> {
                            (self.0).#fn_name(input.into()).map(algorithmia::algo::AlgoOutput::from).map_err(|err| err.into())
                        }
                    }
                    impl Default for Algo {
                        fn default() -> Self {
                            Algo(Default::default())
                        }
                    }

                    #item
                }
            }
            None => {
                quote! {
                    #[derive(Default)] pub struct Algo;
                    impl algorithmia::entrypoint::EntryPoint for Algo {
                        fn #apply_fn(&mut self, input: #input_type) -> ::std::result::Result<algorithmia::algo::AlgoIo, Box<::std::error::Error>> {
                            #fn_name(input.into()).map(algorithmia::algo::AlgoIo::from).map_err(|err| err.into())
                        }
                    }

                    #item
                }
            }
        }
    }

    fn impl_decoded_entrypoint(&self) -> Tokens {
        let ref fn_name = self.fn_name;
        let ref input_type = self.input_type;
        let ref item = self.item;

        match self.self_type {
            Some(ref self_type) => {
                quote! {
                    pub struct Algo(#self_type);
                    impl algorithmia::entrypoint::DecodedEntryPoint for Algo {
                        type Input = #input_type;
                        fn apply_decoded(&mut self, input: #input_type) -> ::std::result::Result<algorithmia::algo::AlgoOutput, Box<::std::error::Error>> {
                            (self.0).#fn_name(input).map(algorithmia::algo::AlgoOutput::from).map_err(|err| err.into())
                        }
                    }

                    impl Default for Algo {
                        fn default() -> Self {
                            Algo(Default::default())
                        }
                    }

                    #item
                }
            }
            None => {
                quote! {
                    #[derive(Default)] pub struct Algo;
                    impl algorithmia::entrypoint::DecodedEntryPoint for Algo {
                        type Input = #input_type;
                        fn apply_decoded(&mut self, input: #input_type) -> ::std::result::Result<algorithmia::algo::AlgoIo, Box<::std::error::Error>> {
                            #fn_name(input).map(algorithmia::algo::AlgoIo::from).map_err(|err| err.into())
                        }
                    }

                    #item
                }
            }
        }
    }
}

fn assert_generics_empty(generics: &Generics) {
    assert!(
        generics.lifetimes.is_empty()
            && generics.ty_params.is_empty()
            && generics.where_clause.predicates.is_empty(),
        "Generics are not supported on the #[algo_entrypoint] function"
    )
}
