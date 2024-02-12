mod dist_iter;
mod function_registry;
mod task;
mod universe_guard;

pub use dist_iter_macros::main;
#[doc(hidden)]
pub use linkme;
#[doc(hidden)]
pub use mpi;

pub use crate::dist_iter::IntoDistIter;
#[doc(hidden)]
pub use crate::{
    function_registry::{tag_to_execute, RegistryEntry, FUNCTION_REGISTRY},
    task::Task,
    universe_guard::UniverseGuard,
};
