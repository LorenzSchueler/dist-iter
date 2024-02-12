use mpi::{topology::SimpleCommunicator, traits::*};

mod dispatch;
mod functions;
mod traits;
mod universe_guard;

use crate::{
    dispatch::{tag_to_execute, tag_to_receive},
    functions::{FibonacciTask, SquareTask},
    traits::Task,
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
    let mut work_queue = (100..120)
        .into_iter()
        .map(|t| Box::new(SquareTask::new(t)) as Box<dyn Task>)
        .chain(
            (0..10)
                .into_iter()
                .map(|t| Box::new(FibonacciTask::new(t)) as Box<dyn Task>),
        )
        .collect::<Vec<_>>();
    let mut recv_queue = Vec::new();
    let total = work_queue.len();

    for dest in 1..world.size() {
        if let Some(task) = work_queue.pop() {
            task.send(world.process_at_rank(dest));
        }
    }

    while recv_queue.len() < total {
        let (msg, status) = world.any_process().matched_probe();

        let receive = tag_to_receive(status.tag());
        recv_queue.push(receive(msg));

        if let Some(task) = work_queue.pop() {
            task.send(world.process_at_rank(status.source_rank()));
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
