use std::ops::DerefMut;

use dist_iter::{map_chunk_task, map_task, DistIterator};

#[dist_iter::main]
fn main() {
    let mut results: Vec<_> = [1, 2, 3, 4, 5]
        .dist_map(map_task!(2, |x: i32| -> i32 { x * x }))
        .collect();
    results.sort();

    eprintln!("{results:?}");
    assert_eq!(results, [1, 4, 9, 16, 25]);

    // map_iter
    let mut results: Vec<_> = [1, 2, 3, 4, 5]
        .dist_map_chunk(map_chunk_task!(
            |iter: UninitBuffer<i32, 2>| -> impl IntoIterator<Item = i32> { iter.map(|x| x * x) }
        ))
        .collect();
    results.sort();

    eprintln!("{results:?}");
    assert_eq!(results, [1, 4, 9, 16, 25]);

    // map_iter with multiple adapters inside
    let mut results: Vec<_> = [1, 2, 3, 4, 5]
        .dist_map_chunk(map_chunk_task!(
            |iter: UninitBuffer<i32, 2>| -> impl IntoIterator<Item = i32> {
                iter.map(|x| x * x).filter(|x| x % 2 == 0)
            }
        ))
        .collect();
    results.sort();

    eprintln!("{results:?}");
    assert_eq!(results, [4, 16]);

    // map_iter with multiple adapters inside and single return value
    let results = [1, 2, 3, 4, 5]
        .dist_map_chunk(map_chunk_task!(
            |iter: UninitBuffer<i32, 2>| -> impl IntoIterator<Item = i32> {
                let sum = iter.map(|x| x * x).filter(|x| x % 2 == 0).sum::<i32>();
                std::iter::once(sum)
            }
        ))
        .sum::<i32>();

    eprintln!("{results:?}");
    assert_eq!(results, 20);

    // TODO
    //// map_iter with more items in send_buf than in recv_buf
    //let results = [1, 2, 3, 4, 5]
    //.dist_map_chunk(map_chunk_task!(
    //|iter: UninitBuffer<i32, 2>| -> impl IntoIterator<Item = i32> { iter.chain(Some(6)) }
    //))
    //.sum::<i32>();

    //eprintln!("{results:?}");
    //assert_eq!(results, 20);

    // map_iter with use of DerefMut
    let mut results: Vec<_> = [1, 2, 3, 4, 5]
        .dist_map_chunk(map_chunk_task!(|buf: &mut UninitBuffer<i32, 2>| {
            for item in buf.deref_mut() {
                *item += 1;
            }
        }))
        .collect();
    results.sort();

    eprintln!("{results:?}");
    assert_eq!(results, [2, 3, 4, 5, 6]);

    //// map_iter with multiple adapters inside
    //let mut results: Vec<_> = [1, 2, 3, 4, 5]
    //.into_dist_iter(worker_task!(2, i32, i32, |iter| {
    //iter.map(|x| x * x).filter(|x| x % 2 == 0)
    //}))
    ////.into_seq_iter()
    //.collect();
    //results.sort();

    //eprintln!("{results:?}");
    //assert_eq!(results, [4, 16]);
}
