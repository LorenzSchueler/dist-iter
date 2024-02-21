use dist_iter::{reduce_task, DistIterator};

#[dist_iter::main]
fn main() {
    let result = [1, 2, 3, 4, 5].dist_reduce(reduce_task!(2, |x: i32, y| { x + y }));

    eprintln!("{result:?}");
    assert_eq!(result, Some(15));

    let result = [1].dist_reduce(reduce_task!(2, |x: i32, y: i32| { x + y }));

    eprintln!("{result:?}");
    assert_eq!(result, Some(1));

    let result = [].dist_reduce(reduce_task!(2, |x: i32, y: i32| -> i32 { x + y }));

    eprintln!("{result:?}");
    assert_eq!(result, None);
}
