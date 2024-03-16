use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicI32, Ordering},
        LazyLock, RwLock,
    },
};

use linkme::distributed_slice;
use mpi::{point_to_point::Message, Tag};
use tracing::trace;

use crate::function_registry::{TaskId, TaskInstanceId, TaskInstanceMapping};

#[doc(hidden)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkerMode {
    Continue,
    Terminate,
}

impl WorkerMode {
    pub fn is_terminate(self) -> bool {
        self == WorkerMode::Terminate
    }
}

#[doc(hidden)]
pub type RegistryFn = fn(Message) -> WorkerMode;

#[doc(hidden)]
pub struct RegistryEntry {
    task_id: TaskId,
    registry_fn: RegistryFn,
}

impl RegistryEntry {
    pub const fn new(task_id: TaskId, registry_fn: RegistryFn) -> Self {
        Self {
            task_id,
            registry_fn,
        }
    }

    fn task_id(&self) -> TaskId {
        self.task_id
    }

    fn registry_fn(&self) -> RegistryFn {
        self.registry_fn
    }
}

#[doc(hidden)]
#[distributed_slice]
pub static FUNCTION_REGISTRY: [RegistryEntry];

pub(crate) const SHUTDOWN_TASK_ID: TaskInstanceId = TaskInstanceId::new(0);

fn shutdown(msg: Message) -> WorkerMode {
    trace!(target: "dist_iter::shutdown_task", "receiving shutdown message ...");
    let mut buf: [u8; 0] = [];
    msg.matched_receive_into(&mut buf);
    trace!(target: "dist_iter::shutdown_task", "received shutdown message");
    trace!(target: "dist_iter::shutdown_task", "indicating shutdown");
    WorkerMode::Terminate
}

pub(crate) const REGISTER_TASK_ID: TaskInstanceId = TaskInstanceId::new(1);

fn register_incoming_task(msg: Message) -> WorkerMode {
    trace!(target: "dist_iter::register_task", "receiving task mapping ...");
    let task_instance_mapping: TaskInstanceMapping = msg.matched_receive().0;
    trace!(target: "dist_iter::register_task", "received task mapping");
    let func = task_id_to_function(task_instance_mapping.task_id());
    FUNCTION_LOOKUP_TABLE
        .write()
        .unwrap()
        .insert(task_instance_mapping.task_instance_id(), func);
    trace!(
        target: "dist_iter::register_task",
        "registered task mapping {} -> {}",
        task_instance_mapping.task_instance_id(),
        task_instance_mapping.task_id()
    );
    WorkerMode::Continue
}

static FUNCTION_LOOKUP_TABLE: LazyLock<RwLock<HashMap<TaskInstanceId, RegistryFn>>> =
    LazyLock::new(|| {
        let mut map = HashMap::<_, RegistryFn>::new();
        map.insert(SHUTDOWN_TASK_ID, shutdown);
        map.insert(REGISTER_TASK_ID, register_incoming_task);
        RwLock::new(map)
    });

static NEXT_TASK_INSTANCE_ID: AtomicI32 = AtomicI32::new(2); // 0 = terminate, 1 = register new TaskInstanceId

fn task_id_to_function(task_id: TaskId) -> RegistryFn {
    FUNCTION_REGISTRY
        .iter()
        .find(|entry| entry.task_id() == task_id)
        .map(|entry| entry.registry_fn())
        .unwrap()
}

pub(crate) fn register_new_task(task_id: TaskId) -> TaskInstanceId {
    let task_instance_id =
        TaskInstanceId::new(NEXT_TASK_INSTANCE_ID.fetch_add(1, Ordering::SeqCst));
    let func = task_id_to_function(task_id);
    FUNCTION_LOOKUP_TABLE
        .write()
        .unwrap()
        .insert(task_instance_id, func);
    task_instance_id
}

#[doc(hidden)]
pub fn task_instance_id_to_function(task_instance_id: TaskInstanceId) -> RegistryFn {
    *FUNCTION_LOOKUP_TABLE
        .read()
        .unwrap()
        .get(&task_instance_id)
        .unwrap()
}

pub(crate) fn check_registry() {
    let mut sorted_ids = FUNCTION_REGISTRY
        .iter()
        .map(|entry| entry.task_id())
        .collect::<Vec<_>>();
    sorted_ids.sort();
    for i in 0..(sorted_ids.len() - 1) {
        if sorted_ids[i] == sorted_ids[i + 1] {
            panic!(
                "task ids are not unique: task id {} exists at least twice",
                sorted_ids[i]
            );
        }
    }
}

/// Generate TaskId.
//
/// Uniqueness is not guaranteed at compile time but check at runtime.
#[doc(hidden)]
pub const fn gen_task_id(file: &str, line: u32, column: u32) -> TaskId {
    let file_hash: [u8; 20] = const_sha1::sha1(file.as_bytes()).as_bytes();
    let tag: u32 = ((line << 16) + column)
        ^ (((file_hash[0] as u32) << 24)
            + ((file_hash[1] as u32) << 16)
            + ((file_hash[2] as u32) << 8)
            + (file_hash[3] as u32))
        ^ (((file_hash[4] as u32) << 24)
            + ((file_hash[5] as u32) << 16)
            + ((file_hash[6] as u32) << 8)
            + (file_hash[7] as u32))
        ^ (((file_hash[8] as u32) << 24)
            + ((file_hash[9] as u32) << 16)
            + ((file_hash[10] as u32) << 8)
            + (file_hash[11] as u32))
        ^ (((file_hash[12] as u32) << 24)
            + ((file_hash[13] as u32) << 16)
            + ((file_hash[14] as u32) << 8)
            + (file_hash[15] as u32));

    TaskId::new((tag as Tag).abs())
}
