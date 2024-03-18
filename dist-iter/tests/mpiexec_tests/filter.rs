use std::sync::atomic::AtomicI32;

use dist_iter::{filter_task, DistIterator};

static LOCAL_COUNT: AtomicI32 = AtomicI32::new(0);

#[test]
#[dist_iter::main]
fn main() {
    let mut results: Vec<_> = [1, 2, 3, 4, 5]
        .into_iter()
        .dist_filter(filter_task!(CHUNK_SIZE = 2, |x: &i32| { x % 2 == 0 }))
        .collect();
    results.sort();

    eprintln!("{results:?}");
    assert_eq!(results, [2, 4]);

    let mut results = [1, 2, 3, 4, 5]
        .into_iter()
        .dist_filter_collect(filter_task!(CHUNK_SIZE = 2, |x: &i32| { x % 2 == 0 }));
    results.sort();

    eprintln!("{results:?}");
    assert_eq!(results, [2, 4]);

    let mut results: Vec<_> = [1, 2, 3, 4, 5]
        .into_iter()
        .dist_filter(filter_task!(CHUNK_SIZE = 2, |_x: &i32| -> bool {
            if LOCAL_COUNT.load(std::sync::atomic::Ordering::SeqCst) == 0 {
                LOCAL_COUNT.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                true
            } else {
                false
            }
        }))
        .collect();
    results.sort();

    eprintln!("{results:?}");
    assert_eq!(results, [1, 3, 5]);
}
