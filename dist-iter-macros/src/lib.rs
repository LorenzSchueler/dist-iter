use proc_macro::TokenStream;
use quote::quote;
use syn::{ItemFn, MetaNameValue};

#[proc_macro_attribute]
pub fn main(args: TokenStream, item: TokenStream) -> TokenStream {
    let input: ItemFn = syn::parse2(item.into()).unwrap();

    if input.sig.ident != "main" {
        panic!("only the main function can be annotated with `dist_iter::main`")
    }
    if !input.sig.inputs.is_empty() {
        panic!("the main function cannot accept arguments")
    }
    if args.is_empty() {
        let main_inner = input.block;
        quote!(
            fn main() -> ::std::process::ExitCode {
                ::dist_iter::main(master)
            }

            fn master() {
                #main_inner
            }
        )
        .into()
    } else {
        let args: MetaNameValue = syn::parse2(args.into()).unwrap();
        if !args.path.is_ident("setup") {
            panic!("expected key `setup`")
        }
        let setup = args.value;
        let main_inner = input.block;
        quote!(
            fn main() -> ::std::process::ExitCode {
                let _: fn() = #setup; // for better compiler error if setup takes arguments
                #setup();
                ::dist_iter::main(master)
            }

            fn master() {
                #main_inner
            }
        )
        .into()
    }
}

#[proc_macro_attribute]
pub fn test(_args: TokenStream, item: TokenStream) -> TokenStream {
    let input: ItemFn = syn::parse2(item.into()).unwrap();

    if !input.sig.inputs.is_empty() {
        panic!("the test function cannot accept arguments")
    } else {
        let fn_name = input.sig.ident;
        let main_inner = input.block;
        quote!(
            fn #fn_name() -> ::std::process::ExitCode {
                ::dist_iter::main(master)
            }

            fn master() {
                #main_inner
            }
        )
        .into()
    }
}
