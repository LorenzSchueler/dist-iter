use dist_iter::{map_task, DistIterator, IntoDistIterator};

#[dist_iter::main]
fn main() {
    let mut results: Vec<_> = [1, 2, 3, 4, 5]
        .into_dist_iter::<2>()
        .map(map_task!(i32, i32, |x| x * x))
        .collect();
    results.sort();

    assert_eq!(results, [1, 4, 9, 16, 25]);
}
