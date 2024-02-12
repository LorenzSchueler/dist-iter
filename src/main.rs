use mpi::{topology::SimpleCommunicator, traits::*};

mod dist_iter;
mod function_registry;
mod task;
mod universe_guard;

use crate::{
    dist_iter::MyIterExt, function_registry::tag_to_execute, task::task,
    universe_guard::UniverseGuard,
};

#[dist_iter_macros::main]
fn main() {
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
