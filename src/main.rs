use mpi::{topology::SimpleCommunicator, traits::*};

mod dispatch;
mod macros;
mod traits;
mod universe_guard;

use crate::{
    dispatch::{tag_to_execute, tag_to_receive},
    macros::task,
    universe_guard::UniverseGuard,
};

fn main() {
    let universe = UniverseGuard::new(mpi::initialize().unwrap());
    let world = universe.world();

    if world.rank() == 0 {
        master(&world);
    } else {
        worker(&world);
    }
}

fn master(world: &SimpleCommunicator) {
    let mut work_queue = (0..10)
        .into_iter()
        .map(task!(2, i32, i32, |x| x * x))
        .chain((0..10).into_iter().map(task!(1, u8, u8, |x| x * 2)));
    let mut recv_queue = Vec::new();
    let mut send_count = 0;
    for dest in 1..world.size() {
        if let Some(task) = work_queue.next() {
            task.send(world.process_at_rank(dest));
            send_count += 1;
        }
    }

    while recv_queue.len() < send_count {
        let (msg, status) = world.any_process().matched_probe();

        let receive = tag_to_receive(status.tag());
        recv_queue.push(receive(msg));

        if let Some(task) = work_queue.next() {
            task.send(world.process_at_rank(status.source_rank()));
            send_count += 1;
        }
    }
}

fn worker(world: &SimpleCommunicator) {
    loop {
        let (msg, status) = world.any_process().matched_probe();

        let execute = tag_to_execute(status.tag());
        let stop = execute(msg, world.process_at_rank(0));
        if stop {
            break;
        }
    }
}
