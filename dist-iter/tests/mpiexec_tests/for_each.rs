use dist_iter::{for_each_task, DistIterator};

#[test]
#[dist_iter::test]
fn for_each() {
    [1, 2, 3, 4, 5]
        .into_iter()
        .dist_for_each(for_each_task!(CHUNK_SIZE = 2, |x: i32| {
            println!("{x}");
        }));
}
