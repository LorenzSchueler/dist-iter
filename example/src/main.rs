use dist_iter::{map_task, DistIterator};

#[dist_iter::main]
fn main() {
    let mut results: Vec<_> = [1, 2, 3, 4, 5]
        .into_iter()
        .dist_map(map_task!(CHUNK_SIZE = 2, |x: i32| -> i32 { x * x }))
        .collect();
    results.sort();

    eprintln!("{results:?}");
    assert_eq!(results, [1, 4, 9, 16, 25]);
}
