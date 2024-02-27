use dist_iter::{map_task, DistIterator};

#[dist_iter::main]
fn main() {
    // ...
    let _ = [1, 2, 3, 4, 5]
        .into_iter()
        // possibly more adapters
        .dist_map(map_task!(CHUNK_SIZE = 2, |x: i32| -> i32 {
            // very expensive operation
            x * x
        }))
        // possibly more adapters
        .collect::<Vec<_>>();
    // ...
}
