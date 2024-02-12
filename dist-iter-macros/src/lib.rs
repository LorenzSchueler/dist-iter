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
                let universe = UniverseGuard::new(mpi::initialize().unwrap());
                let world = universe.world();

                if world.rank() == 0 {
                    master(&world);
                } else {
                    worker(&world);
                }
            }

            fn master(world: &SimpleCommunicator) {
                #main_inner
            }

            fn worker(world: &SimpleCommunicator) {
                loop {
                    let (msg, status) = world.any_process().matched_probe();

                    let execute = tag_to_execute(status.tag());
                    let stop = execute(msg, world.process_at_rank(0));
                    if stop {
                        break;
                    }
                }
            }
        )
        .into()
    }
}
