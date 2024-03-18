use std::{thread, time::Duration};

use dist_iter::{map_task, DistIterator};

#[test]
#[dist_iter::main]
fn main() {
    let t1 = thread::spawn(|| run(0));
    let t2 = thread::spawn(|| run(20));
    t1.join().unwrap();
    t2.join().unwrap();
}

fn run(id: i32) {
    let mut results: Vec<_> = (id..=id + 20)
        .dist_map(map_task!(CHUNK_SIZE = 2, |x: i32| -> i32 {
            // sleep a little bit to make sure execution of both threads is interleaved
            thread::sleep(Duration::from_millis(100));
            x
        }))
        .collect();
    results.sort();

    eprintln!("{results:?}");
    assert_eq!(results, (id..=id + 20).collect::<Vec<_>>());
}
