use dist_iter::{map_task, DistIterator};
use mpi::traits::Equivalence;
use tracing_subscriber::filter::LevelFilter;

#[derive(Equivalence, Debug)]
struct Wrapper {
    x: i32,
}

#[dist_iter::main(setup = setup)]
fn main() {
    let results: Vec<_> = [1, 2, 3, 4, 5]
        .into_iter()
        .dist_map(map_task!(CHUNK_SIZE = 2, |x: i32| -> Wrapper {
            Wrapper { x }
        }))
        .collect();

    eprintln!("{results:?}");
}

fn setup() {
    tracing_subscriber::fmt()
        .with_max_level(LevelFilter::TRACE)
        .init();
}
