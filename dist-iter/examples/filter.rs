use dist_iter::{filter_task, DistIterator, IntoDistIterator};

#[dist_iter::main]
fn main() {
    let mut results: Vec<_> = [1, 2, 3, 4, 5]
        .into_dist_iter::<2>()
        .filter(filter_task!(2, i32, |x| x % 2 == 0))
        .collect();
    results.sort();

    eprintln!("{results:?}");
    assert_eq!(results, [2, 4]);
}
