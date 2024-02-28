use dist_iter::{map_chunk_task, DistIterator};

fn main() {
    [1, 2, 3, 4, 5]
        .into_iter()
        .dist_map_chunk(map_chunk_task!(CHUNK_SIZE = 2, |buf| {
            for item in buf {
                *item += 1;
            }
        }));
}
