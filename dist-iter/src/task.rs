use mpi::traits::Equivalence;

use crate::TaskId;

#[doc(hidden)]
pub trait Task {
    type In: Equivalence;
    type Out: Equivalence;

    const IN: usize;
    const OUT: usize;
    const ID: TaskId;
}

#[doc(hidden)]
pub struct MapChunkTask<T: Task> {
    pub task: T,
}

#[doc(hidden)]
pub struct MapTask<T: Task> {
    pub task: T,
}

#[doc(hidden)]
pub struct FilterTask<T: Task> {
    pub task: T,
}

#[doc(hidden)]
pub struct ReduceTask<T: Task> {
    pub task: T,
}

#[doc(hidden)]
pub struct ForEachTask<T: Task> {
    pub task: T,
}

#[doc(hidden)]
#[macro_export]
macro_rules! register_execute_and_return_task {
    ($in:ty, $out:ty, $IN:literal, $OUT:literal, $ID:expr) => {{
        #[::dist_iter::linkme::distributed_slice(::dist_iter::FUNCTION_REGISTRY)]
        #[linkme(crate = ::dist_iter::linkme)]
        static REGISTRY_ENTRY: ::dist_iter::RegistryEntry =
            ::dist_iter::RegistryEntry::new(ID, execute);

        struct ThisTask {}

        impl ::dist_iter::Task for ThisTask {
            type In = $in;
            type Out = $out;

            const IN: usize = $IN;
            const OUT: usize = $OUT;
            const ID: ::dist_iter::TaskId = ID;
        }

        ThisTask {}
    }};
}

#[doc(hidden)]
#[macro_export]
macro_rules! task {
    (INPUT_CHUNK_SIZE = $IN:literal, OUTPUT_CHUNK_SIZE = $OUT:literal, |$closure_param:ident: impl Iterator<Item = $in:ty>| -> impl IntoIterator<Item = $out:ty> $closure_block:block) => {{
        #[inline(always)]
        fn function($closure_param: impl Iterator<Item = $in>) -> impl IntoIterator<Item = $out> {
            $closure_block
        }

        fn execute(msg: ::dist_iter::mpi::point_to_point::Message) -> ::dist_iter::WorkerMode {
            use ::dist_iter::mpi::{
                point_to_point::Destination,
                topology::{Communicator, SimpleCommunicator},
            };

            ::dist_iter::tracing::trace!(target: "dist_iter::task", "receiving data ...");
            let (recv_buf, task_instance_id) = ::dist_iter::UninitBuffer::<_, $IN>::from_matched_receive(msg);
            ::dist_iter::tracing::trace!(target: "dist_iter::task", "received data of length {}", recv_buf.len());
            let result = function(recv_buf);

            let mut send_buf = ::dist_iter::UninitBuffer::<_, $OUT>::new();
            for item in result {
                send_buf.push_back_unchecked(item);
            }
            ::dist_iter::tracing::trace!(target: "dist_iter::task", "sending response of length {} ...", send_buf.len());
            SimpleCommunicator::world()
                .process_at_rank(::dist_iter::MASTER)
                .send_with_tag(&*send_buf, *task_instance_id);
            ::dist_iter::tracing::trace!(target: "dist_iter::task", "response sent");

            ::dist_iter::WorkerMode::Continue
        }

        const ID: ::dist_iter::TaskId = ::dist_iter::gen_task_id(file!(), line!(), column!());

        ::dist_iter::register_execute_and_return_task!($in, $out, $IN, $OUT, ID)
    }};
    (CHUNK_SIZE = $IN:literal, |$closure_param:ident: &mut [$in:ty]| $closure_block:block) => {{
        #[inline(always)]
        fn function($closure_param: &mut [$in]) {
            $closure_block
        }

        fn execute(msg: ::dist_iter::mpi::point_to_point::Message) -> ::dist_iter::WorkerMode {
            use ::dist_iter::mpi::{
                point_to_point::Destination,
                topology::{Communicator, SimpleCommunicator},
            };

            ::dist_iter::tracing::trace!(target: "dist_iter::task", "receiving data ...");
            let (mut buf, task_instance_id) = ::dist_iter::UninitBuffer::<_, $IN>::from_matched_receive(msg);
            ::dist_iter::tracing::trace!(target: "dist_iter::task", "received data of length {}", buf.len());
            function(&mut buf);

            ::dist_iter::tracing::trace!(target: "dist_iter::task", "sending response of length {} ...", buf.len());
            SimpleCommunicator::world()
                .process_at_rank(::dist_iter::MASTER)
                .send_with_tag(&*buf, *task_instance_id);
            ::dist_iter::tracing::trace!(target: "dist_iter::task", "response sent");

            ::dist_iter::WorkerMode::Continue
        }

        const ID: ::dist_iter::TaskId = ::dist_iter::gen_task_id(file!(), line!(), column!());

        ::dist_iter::register_execute_and_return_task!($in, $in, $IN, $IN, ID)
    }};
}

#[doc(hidden)]
#[macro_export]
macro_rules! no_response_task {
    (INPUT_CHUNK_SIZE = $IN:literal, |$closure_param:ident: impl Iterator<Item = $in:ty>| $closure_block:block) => {{
        #[inline(always)]
        fn function($closure_param: impl Iterator<Item = $in>) {
            $closure_block
        }

        fn execute(msg: ::dist_iter::mpi::point_to_point::Message) -> ::dist_iter::WorkerMode {
            use ::dist_iter::mpi::{
                point_to_point::Destination,
                topology::{Communicator, SimpleCommunicator},
            };

            ::dist_iter::tracing::trace!(target: "dist_iter::task", "receiving data ...");
            let (recv_buf, task_instance_id) = ::dist_iter::UninitBuffer::<_, $IN>::from_matched_receive(msg);
            ::dist_iter::tracing::trace!(target: "dist_iter::task", "received data of length {}", recv_buf.len());
            function(recv_buf);

            let send_buf: [u8; 0] = [];
            ::dist_iter::tracing::trace!(target: "dist_iter::task", "sending response of length {} ...", send_buf.len());
            SimpleCommunicator::world()
                .process_at_rank(::dist_iter::MASTER)
                .send_with_tag(&send_buf, *task_instance_id);
            ::dist_iter::tracing::trace!(target: "dist_iter::task", "response sent");

            ::dist_iter::WorkerMode::Continue
        }

        const ID: ::dist_iter::TaskId = ::dist_iter::gen_task_id(file!(), line!(), column!());

        ::dist_iter::register_execute_and_return_task!($in, u8, $IN, 0, ID)
    }};
}

#[macro_export]
macro_rules! map_chunk_task {
    ($($tree:tt)+) => {{
        ::dist_iter::MapChunkTask{
            task: ::dist_iter::task!($($tree)+)
        }
    }};
}

#[macro_export]
macro_rules! map_task {
    (CHUNK_SIZE = $IN:literal, |$closure_param:ident: $in:ty| -> $out:ty $closure_block:block) => {{
        ::dist_iter::MapTask {
            task: ::dist_iter::task!(
                INPUT_CHUNK_SIZE = $IN,
                OUTPUT_CHUNK_SIZE = $IN,
                |iter: impl Iterator<Item = $in>| -> impl IntoIterator<Item = $out> {
                    iter.map(|$closure_param: $in| $closure_block)
                }
            ),
        }
    }};
}

#[macro_export]
macro_rules! filter_task {
    (CHUNK_SIZE = $IN:literal, |$closure_param:ident: &$in:ty| $(-> bool)? $closure_block:block) => {{
        ::dist_iter::FilterTask {
            task: ::dist_iter::task!(
                INPUT_CHUNK_SIZE = $IN,
                OUTPUT_CHUNK_SIZE = $IN,
                |iter: impl Iterator<Item = $in>| -> impl IntoIterator<Item = $in> {
                    iter.filter(|$closure_param: &$in| $closure_block)
                }
            ),
        }
    }};
}

#[macro_export]
macro_rules! reduce_task {
    // TODO make sure in == in2 == in3
    (CHUNK_SIZE = $IN:literal, |$closure_param1:ident: $in:ty, $closure_param2:ident $(:$in2:ty)?| $(-> $in3:ty)? $closure_block:block) => {{
        (
            ::dist_iter::ReduceTask {
                task: ::dist_iter::task!(
                    INPUT_CHUNK_SIZE = $IN,
                    OUTPUT_CHUNK_SIZE = 1,
                    |iter: impl Iterator<Item = $in>| -> impl IntoIterator<Item = $in> {
                        iter.reduce(|$closure_param1: $in, $closure_param2| $closure_block)
                    }
                ),
            },
            |$closure_param1: $in, $closure_param2| $closure_block,
        )
    }};
}

#[macro_export]
macro_rules! for_each_task {
    (CHUNK_SIZE = $IN:literal, |$closure_param:ident: $in:ty| $closure_block:block) => {{
        ::dist_iter::ForEachTask {
            task: ::dist_iter::no_response_task! {
                INPUT_CHUNK_SIZE = $IN,
                |iter: impl Iterator<Item = $in>| {
                iter.for_each(|$closure_param: $in| $closure_block);
            }},
        }
    }};
}
