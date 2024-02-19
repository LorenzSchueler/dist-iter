use std::sync::atomic::AtomicI32;

use dist_iter::{filter_task, DistIterator, IntoDistIterator};

static LOCAL_COUNT: AtomicI32 = AtomicI32::new(0);

#[dist_iter::main]
fn main() {
    let mut results: Vec<_> = [1, 2, 3, 4, 5]
        .into_dist_iter::<2>()
        .filter(filter_task!(2, i32, |x| x % 2 == 0))
        .collect();
    results.sort();

    eprintln!("{results:?}");
    assert_eq!(results, [2, 4]);

    let mut results: Vec<_> = [1, 2, 3, 4, 5]
        .into_dist_iter::<2>()
        .filter(filter_task!(2, i32, |_x| {
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
