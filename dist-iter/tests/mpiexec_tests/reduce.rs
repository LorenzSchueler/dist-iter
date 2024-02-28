use dist_iter::{reduce_task, DistIterator};

#[test]
#[dist_iter::test]
fn reduce() {
    let result = [1, 2, 3, 4, 5]
        .into_iter()
        .dist_reduce(reduce_task!(CHUNK_SIZE = 2, |x: i32, y| { x + y }));

    eprintln!("{result:?}");
    assert_eq!(result, Some(15));

    let result = [1]
        .into_iter()
        .dist_reduce(reduce_task!(CHUNK_SIZE = 2, |x: i32, y: i32| { x + y }));

    eprintln!("{result:?}");
    assert_eq!(result, Some(1));

    let result = []
        .into_iter()
        .dist_reduce(reduce_task!(CHUNK_SIZE = 2, |x: i32, y: i32| -> i32 {
            x + y
        }));

    eprintln!("{result:?}");
    assert_eq!(result, None);
}
