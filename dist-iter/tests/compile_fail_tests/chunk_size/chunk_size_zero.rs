use dist_iter::{map_task, DistIterator};

fn main() {
    let _ = []
        .into_iter()
        .dist_map(map_task!(CHUNK_SIZE = 0, |x: i32| -> i32 { x * x }));
}
