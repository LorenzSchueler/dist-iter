use proc_macro::TokenStream;
use quote::quote;
use syn::ItemFn;

#[proc_macro_attribute]
pub fn main(_args: TokenStream, item: TokenStream) -> TokenStream {
    let input: ItemFn = syn::parse2(item.into()).unwrap();

    if input.sig.ident == "main" && !input.sig.inputs.is_empty() {
        panic!("the main function cannot accept arguments")
    } else {
        let main_inner = input.block;
        quote!(
            fn main() {
                ::dist_iter::main(master)
            }

            fn master(world: &::dist_iter::mpi::topology::SimpleCommunicator) {
                #main_inner
            }
        )
        .into()
    }
}
