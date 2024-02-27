use dist_iter::{map_task, DistIterator};
use mpi::traits::Equivalence;

#[derive(Equivalence, Debug)]
struct Wrapper {
    x: i32,
}

#[dist_iter::main]
fn main() {
    let results: Vec<_> = [1, 2, 3, 4, 5]
        .into_iter()
        .dist_map(map_task!(CHUNK_SIZE = 2, |x: i32| -> Wrapper {
            Wrapper { x }
        }))
        .collect();

    eprintln!("{results:?}");
}
