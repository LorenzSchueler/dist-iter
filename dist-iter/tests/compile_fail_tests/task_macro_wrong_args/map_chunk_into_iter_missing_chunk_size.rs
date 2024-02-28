use dist_iter::{map_chunk_task, DistIterator};

fn main() {
    [1, 2, 3, 4, 5].into_iter().dist_map_chunk(map_chunk_task!(
        |buf: impl Iterator<Item = i32>| -> impl IntoIterator<Item = i32> { buf.map(|x| x + 1) }
    ));
}
