use std::sync::atomic::{AtomicBool, Ordering};

use dist_iter::{map_task, DistIterator};

static SETUP_RUN: AtomicBool = AtomicBool::new(false);

#[test]
#[dist_iter::main(setup = setup)]
fn main() {
    if !SETUP_RUN.load(Ordering::SeqCst) {
        panic!("master did not run setup")
    }

    let workers_setup_run = [1, 2, 3, 4]
        .into_iter()
        .dist_map(map_task!(CHUNK_SIZE = 1, |_x: i32| -> bool {
            SETUP_RUN.load(Ordering::SeqCst)
        }))
        .all(|setup_run| setup_run);

    if !workers_setup_run {
        panic!("workers did not run setup")
    }
}

fn setup() {
    SETUP_RUN.store(true, Ordering::SeqCst)
}
