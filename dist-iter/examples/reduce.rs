use dist_iter::{reduce_task, DistIterator, IntoDistIterator};

#[dist_iter::main]
fn main() {
    let result = [1, 2, 3, 4, 5]
        .into_dist_iter::<2>()
        .reduce(reduce_task!(2, i32, |x, y| x + y));

    eprintln!("{result:?}");
    assert_eq!(result, Some(15));
}
