use mpi::{topology::SimpleCommunicator, traits::*};

mod dist_iter;
mod function_registry;
mod task;
mod universe_guard;

use crate::{
    dist_iter::MyIterExt, function_registry::tag_to_execute, task::task,
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
    (0..10)
        .into_iter()
        .map(task!(2, i32, i32, |x| x * x))
        .into_dist_iter(world)
        .for_each(|v| println!("{v}"));
    (0..10)
        .into_iter()
        .map(task!(1, u8, u8, |x| x * 2))
        .into_dist_iter(world)
        .for_each(|v| println!("{v}"));
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
