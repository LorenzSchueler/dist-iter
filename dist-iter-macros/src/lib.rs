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
                use dist_iter::mpi::topology::Communicator;

                let universe = dist_iter::UniverseGuard::new(dist_iter::mpi::initialize().unwrap());
                let world = universe.world();

                if world.rank() == 0 {
                    master(&world);
                } else {
                    worker(&world);
                }
            }

            fn master(world: &dist_iter::mpi::topology::SimpleCommunicator) {
                #main_inner
            }

            fn worker(world: &dist_iter::mpi::topology::SimpleCommunicator) {
                use dist_iter::mpi::{topology::Communicator, point_to_point::Source};

                loop {
                    let (msg, status) = world.any_process().matched_probe();

                    let execute = dist_iter::tag_to_execute(status.tag());
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
