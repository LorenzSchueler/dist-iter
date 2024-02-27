use dist_iter::{filter_task, map_chunk_task, map_task, DistIterator};

#[dist_iter::main]
fn main() {
    let my_iter = [].into_iter();
    let _ = my_iter
        .dist_map(map_task!(CHUNK_SIZE = 10, |x: i32| -> i32 { x * x }))
        .dist_filter(filter_task!(CHUNK_SIZE = 10, |x: &i32| { x % 2 == 0 }));

    let my_iter = [].into_iter();
    let _ = my_iter.dist_map_chunk(map_chunk_task!(
        INPUT_CHUNK_SIZE = 2,
        OUTPUT_CHUNK_SIZE = 2,
        |iter: impl Iterator<Item = i32>| -> impl IntoIterator<Item = i32> {
            iter.map(|x| x * x).filter(|x| x % 2 == 0)
        }
    ));
}
