use dist_iter::{map_chunk_task, map_task, DistIterator};

#[dist_iter::main]
fn main() {
    // map
    let mut results: Vec<_> = [1, 2, 3, 4, 5]
        .into_iter()
        .dist_map(map_task!(2, |x: i32| -> i32 { x * x }))
        .collect();
    results.sort();

    eprintln!("{results:?}");
    assert_eq!(results, [1, 4, 9, 16, 25]);

    // map_chunk
    let mut results: Vec<_> = [1, 2, 3, 4, 5]
        .into_iter()
        .dist_map_chunk(map_chunk_task!(
            2,
            |iter: impl Iterator<Item = i32>| -> impl IntoIterator<Item = i32, LEN = 2> {
                iter.map(|x| x * x)
            }
        ))
        .collect();
    results.sort();

    eprintln!("{results:?}");
    assert_eq!(results, [1, 4, 9, 16, 25]);

    // map_chunk with multiple adapters inside
    let mut results: Vec<_> = [1, 2, 3, 4, 5]
        .into_iter()
        .dist_map_chunk(map_chunk_task!(
            2,
            |iter: impl Iterator<Item = i32>| -> impl IntoIterator<Item = i32, LEN = 2> {
                iter.map(|x| x * x).filter(|x| x % 2 == 0)
            }
        ))
        .collect();
    results.sort();

    eprintln!("{results:?}");
    assert_eq!(results, [4, 16]);

    // map_chunk with multiple adapters inside and single return value
    let results = [1, 2, 3, 4, 5]
        .into_iter()
        .dist_map_chunk(map_chunk_task!(
            2,
            |iter: impl Iterator<Item = i32>| -> impl IntoIterator<Item = i32, LEN = 2> {
                let sum = iter.map(|x| x * x).filter(|x| x % 2 == 0).sum::<i32>();
                std::iter::once(sum)
            }
        ))
        .sum::<i32>();

    eprintln!("{results:?}");
    assert_eq!(results, 20);

    // map_chunk with more items in send_buf than in recv_buf
    let mut results: Vec<_> = [1, 2, 3, 4, 5]
        .into_iter()
        .dist_map_chunk(map_chunk_task!(
            2,
            |iter: impl Iterator<Item = i32>| -> impl IntoIterator<Item = i32, LEN = 3> {
                iter.chain(Some(6))
            }
        ))
        .collect();
    results.sort();

    eprintln!("{results:?}");
    assert_eq!(results, [1, 2, 3, 4, 5, 6, 6, 6]);

    // map_chunk with use of DerefMut
    let mut results: Vec<_> = [1, 2, 3, 4, 5]
        .into_iter()
        .dist_map_chunk(map_chunk_task!(2, |buf: &mut [i32]| {
            for item in buf {
                *item += 1;
            }
        }))
        .collect();
    results.sort();

    eprintln!("{results:?}");
    assert_eq!(results, [2, 3, 4, 5, 6]);

    // map_chunk where first result chunks are all empty
    let mut results: Vec<_> = [1, 3, 5, 7, 9, 11, 13, 15, 17, 19, 21, 23, 25, 2, 2, 2, 2]
        .into_iter()
        .dist_map_chunk(map_chunk_task!(
            2,
            |iter: impl Iterator<Item = i32>| -> impl IntoIterator<Item = i32, LEN = 2> {
                iter.filter(|x| x % 2 == 0)
            }
        ))
        .collect();
    results.sort();

    eprintln!("{results:?}");
    assert_eq!(results, [2, 2, 2, 2]);
}
