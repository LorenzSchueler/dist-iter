use dist_iter::{map_iter_task, map_task, DistIterator, IntoDistIterator};

#[dist_iter::main]
fn main() {
    let mut results: Vec<_> = [1, 2, 3, 4, 5]
        .into_dist_iter::<2>()
        .map(map_task!(2, i32, i32, |x| x * x))
        .collect();
    results.sort();

    eprintln!("{results:?}");
    assert_eq!(results, [1, 4, 9, 16, 25]);

    // map_iter
    let mut results: Vec<_> = [1, 2, 3, 4, 5]
        .into_dist_iter::<2>()
        //.map_iter(map_iter_task!(2, i32, i32, |iter| { iter.map(|x| x * x) }))
        .map_iter(map_iter_task!(
            2,
            |iter: impl Iterator<Item = i32>| -> impl IntoIterator<Item = i32> {
                iter.map(|x| x * x)
            }
        ))
        .collect();
    results.sort();

    eprintln!("{results:?}");
    assert_eq!(results, [1, 4, 9, 16, 25]);

    // map_iter with multiple adapters inside
    let mut results: Vec<_> = [1, 2, 3, 4, 5]
        .into_dist_iter::<2>()
        .map_iter(map_iter_task!(
            2,
            |iter: impl Iterator<Item = i32>| -> impl IntoIterator<Item = i32> {
                iter.map(|x| x * x).filter(|x| x % 2 == 0)
            }
        ))
        .collect();
    results.sort();

    eprintln!("{results:?}");
    assert_eq!(results, [4, 16]);

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

    // map_iter with multiple adapters inside and single return value
    let results = [1, 2, 3, 4, 5]
        .into_dist_iter::<2>()
        .map_iter(map_iter_task!(
            2,
            |iter: impl Iterator<Item = i32>| -> impl IntoIterator<Item = i32> {
                let sum = iter.map(|x| x * x).filter(|x| x % 2 == 0).sum::<i32>();
                std::iter::once(sum)
            }
        ))
        .sum::<i32>();

    eprintln!("{results:?}");
    assert_eq!(results, 20);
}
