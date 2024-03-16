use std::ops::Deref;

use mpi::{traits::Equivalence, Tag};

#[doc(hidden)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct TaskId(Tag);

impl TaskId {
    pub const fn new(tag: Tag) -> Self {
        Self(tag)
    }
}

impl std::fmt::Display for TaskId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[doc(hidden)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TaskInstanceId(Tag);

impl TaskInstanceId {
    pub const fn new(tag: Tag) -> Self {
        Self(tag)
    }
}

impl std::fmt::Display for TaskInstanceId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl Deref for TaskInstanceId {
    type Target = Tag;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Equivalence)]
pub(crate) struct TaskInstanceMapping {
    task_id: Tag,
    task_instance_id: Tag,
}

impl TaskInstanceMapping {
    pub fn new(task_id: TaskId, task_instance_id: TaskInstanceId) -> Self {
        Self {
            task_id: task_id.0,
            task_instance_id: task_instance_id.0,
        }
    }

    pub fn task_id(&self) -> TaskId {
        TaskId(self.task_id)
    }

    pub fn task_instance_id(&self) -> TaskInstanceId {
        TaskInstanceId(self.task_instance_id)
    }
}
