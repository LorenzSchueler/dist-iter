use dist_iter::{map_chunk_task, DistIterator};

fn main() {
    [1, 2, 3, 4, 5].into_iter().dist_map_chunk(map_chunk_task!(
        INPUT_CHUNK_SIZE = 2,
        OUTPUT_CHUNK_SIZE = 2,
        |buf: impl Iterator<Item = i32>| { buf.map(|x| x + 1) }
    ));
}
