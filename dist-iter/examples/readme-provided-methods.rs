use dist_iter::{filter_task, map_chunk_task, map_task, reduce_task, DistIterator};

#[dist_iter::main]
fn main() {
    let my_iter = [].into_iter();
    // .dist_map(map_task!(CHUNK_SIZE = <N>, |x: <input type>| -> <output type> { ... }))
    let _ = my_iter.dist_map(map_task!(CHUNK_SIZE = 10, |x: i32| -> i32 { x * x }));

    let my_iter = [].into_iter();
    // .dist_filter(filter_task!(CHUNK_SIZE = <N>, |x: &<type>| { ... }))
    // OR
    // .dist_filter(filter_task!(CHUNK_SIZE = <N>, |x: &<type>| -> bool { ... }))
    let _ = my_iter.dist_filter(filter_task!(CHUNK_SIZE = 10, |x: &i32| { x % 2 == 0 }));

    let my_iter = [].into_iter();
    // .dist_reduce(reduce_task!(CHUNK_SIZE = <N>, |x: <type>, y| { ... }));
    // OR
    // .dist_reduce(reduce_task!(CHUNK_SIZE = <N>, |x: <type>, y: <type>| { ... }));
    // OR
    // .dist_reduce(reduce_task!(CHUNK_SIZE = <N>, |x: <type>, y: <type>| -> <type> { ... }));
    // NOTE: type must be the same for x, y and return value
    let _ = my_iter.dist_reduce(reduce_task!(CHUNK_SIZE = 10, |x: i32, y| { x + y }));

    let my_iter = [].into_iter();
    // .dist_map_chunk(map_chunk_task!(
    //     INPUT_CHUNK_SIZE = <N>,
    //     OUTPUT_CHUNK_SIZE = <N>,
    //     |iter: impl Iterator<Item = <input type>>| -> impl IntoIterator<Item = <output type>> {
    //         ...
    //     }
    // ))
    let _ = my_iter.dist_map_chunk(map_chunk_task!(
        INPUT_CHUNK_SIZE = 5,
        OUTPUT_CHUNK_SIZE = 10,
        |iter: impl Iterator<Item = i32>| -> impl IntoIterator<Item = i32> {
            // return IntoIterator which will return at most LEN items
            iter.map(|x| x * x).filter(|x| x % 2 == 0)
        }
    ));

    let my_iter = [].into_iter();
    // .dist_map_chunk(map_chunk_task!(CHUNK_SIZE = <N>, |buf: &mut [<type>]| {
    //     ...
    // }))
    let _ = my_iter.dist_map_chunk(map_chunk_task!(CHUNK_SIZE = 10, |buf: &mut [i32]| {
        // modify in place
        for item in buf {
            *item += 1;
        }
    }));
}
