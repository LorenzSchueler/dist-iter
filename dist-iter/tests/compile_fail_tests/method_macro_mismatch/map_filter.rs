use dist_iter::{filter_task, DistIterator};

fn main() {
    [1, 2, 3, 4, 5]
        .into_iter()
        .dist_map(filter_task!(CHUNK_SIZE = 2, |x: i32| -> i32 { x + 1 }));
    [1, 2, 3, 4, 5]
        .into_iter()
        .dist_map(filter_task!(CHUNK_SIZE = 2, |x: &i32| -> bool { x == 0 }));
}
