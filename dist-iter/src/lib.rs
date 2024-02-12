mod dist_iter;
mod function_registry;
mod task;
mod universe_guard;

pub use dist_iter_macros::main;
pub use linkme;
pub use mpi;

pub use crate::{
    dist_iter::MyIterExt,
    function_registry::{tag_to_execute, FUNCTION_REGISTRY},
    task::Task,
    universe_guard::UniverseGuard,
};
