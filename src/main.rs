use mpi::{topology::SimpleCommunicator, traits::*};

mod dispatch;
mod functions;
mod traits;
mod universe_guard;

use crate::{
    dispatch::tag_to_function,
    functions::{FibonacciTask, SquareTask},
    traits::Task,
    universe_guard::UniverseGuard,
};

fn main() {
    let universe = UniverseGuard::new(mpi::initialize().unwrap());

    let world = universe.world();
    let size = world.size();
    let rank = world.rank();

    if rank == 0 {
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

        for dest in 1..size {
            if let Some(task) = work_queue.pop() {
                task.send(&world, dest);
            }
        }

        while recv_queue.len() < total {
            let (msg, status) = world.any_process().matched_probe();

            recv_queue.push(tag_to_function(status.tag()).receive(msg));

            if let Some(task) = work_queue.pop() {
                task.send(&world, status.source_rank());
            }
        }
    } else {
        worker(&world);
    }
}

fn worker(world: &SimpleCommunicator) {
    loop {
        let (msg, status) = world.any_process().matched_probe();

        let function = tag_to_function(status.tag());
        let stop = function.execute(msg, world);
        if stop {
            break;
        }
    }
}
