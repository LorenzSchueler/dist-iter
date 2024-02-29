use dist_iter::{map_task, DistIterator};

fn main() {
    [1, 2, 3, 4, 5]
        .into_iter()
        .dist_map(map_task!(CHUNK_SIZE = 2, |x: i32| -> i32 {
            x + 1;
        }));
}
