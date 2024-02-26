use dist_iter::{map_task, DistIterator};
use mpi::traits::Equivalence;

#[dist_iter::main]
fn main() {
    #[derive(Equivalence, Debug)]
    struct Wrapper {
        x: i32,
        y: i32,
    }

    let results: Vec<_> = [1, 2, 3, 4, 5]
        .into_iter()
        .dist_map(map_task!(2, |x: i32| -> Wrapper { Wrapper { x, y: x } }))
        .collect();

    eprintln!("{results:?}");
}
