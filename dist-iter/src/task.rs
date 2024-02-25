use mpi::{traits::Equivalence, Tag};

#[doc(hidden)]
pub trait Task {
    type In: Equivalence;
    type Out: Equivalence;

    const IN: usize;
    const OUT: usize;
    const TAG: Tag;
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
#[macro_export]
macro_rules! register_execute_and_return_task {
    ($in:ty, $out:ty, $IN:literal, $OUT:literal) => {{
        const TAG: ::dist_iter::mpi::Tag = ::dist_iter::gen_tag(file!(), line!(), column!());

        #[linkme::distributed_slice(::dist_iter::FUNCTION_REGISTRY)]
        static REGISTRY_ENTRY: ::dist_iter::RegistryEntry = (TAG, execute);
        struct ThisTask {}

        impl ::dist_iter::Task for ThisTask {
            type In = $in;
            type Out = $in;

            const IN: usize = $IN;
            const OUT: usize = $OUT;
            const TAG: ::dist_iter::mpi::Tag = TAG;
        }

        ThisTask {}
    }};
}

#[doc(hidden)]
#[macro_export]
macro_rules! task {
    (|$closure_param:ident: UninitBuffer<$in:ty, $IN:literal>| -> impl IntoIterator<Item = $out:ty, LEN = $OUT:literal> $closure_block:block) => {{
        fn function(
            $closure_param: ::dist_iter::UninitBuffer<$in, $IN>,
        ) -> impl IntoIterator<Item = $out> {
            $closure_block
        }

        fn execute(msg: ::dist_iter::mpi::point_to_point::Message) -> ::dist_iter::WorkerMode {
            use ::dist_iter::mpi::{
                point_to_point::Destination,
                topology::{Communicator, SimpleCommunicator},
            };

            let (recv_buf, tag) = ::dist_iter::UninitBuffer::<_, $IN>::from_matched_receive(msg);
            eprintln!(
                "    > [{}] data of length {:?}",
                std::process::id(),
                recv_buf.len()
            );
            let result = function(recv_buf);

            let mut send_buf = ::dist_iter::UninitBuffer::<_, $OUT>::new();
            for item in result {
                send_buf.push_back_unchecked(item);
            }
            eprintln!(
                "    < [{}] data of length {:?}",
                std::process::id(),
                send_buf.len()
            );
            SimpleCommunicator::world()
                .process_at_rank(::dist_iter::MASTER)
                .send_with_tag(&*send_buf, tag);

            ::dist_iter::WorkerMode::Continue
        }

        ::dist_iter::register_execute_and_return_task!($in, $out, $IN, $OUT)
    }};
    (|$closure_param:ident: &mut UninitBuffer<$in:ty, $IN:literal>| $closure_block:block) => {{
        fn function($closure_param: &mut ::dist_iter::UninitBuffer<$in, $IN>) {
            $closure_block
        }

        fn execute(msg: ::dist_iter::mpi::point_to_point::Message) -> ::dist_iter::WorkerMode {
            use ::dist_iter::mpi::{
                point_to_point::Destination,
                topology::{Communicator, SimpleCommunicator},
            };

            let (mut buf, tag) = ::dist_iter::UninitBuffer::<_, $IN>::from_matched_receive(msg);
            eprintln!(
                "    > [{}] data of length {:?}",
                std::process::id(),
                buf.len()
            );
            function(&mut buf);

            eprintln!(
                "    < [{}] data of length {:?}",
                std::process::id(),
                buf.len()
            );
            SimpleCommunicator::world()
                .process_at_rank(::dist_iter::MASTER)
                .send_with_tag(&*buf, tag);

            ::dist_iter::WorkerMode::Continue
        }

        ::dist_iter::register_execute_and_return_task!($in, $in, $IN, $IN)
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
    ($IN:literal, |$closure_param:ident: $in:ty| -> $out:ty $closure_block:block) => {{
        ::dist_iter::MapTask {
            task: ::dist_iter::task!(
                |iter: UninitBuffer<$in, $IN>| -> impl IntoIterator<Item = $out, LEN = $IN> {
                    iter.map(|$closure_param: $in| $closure_block)
                }
            ),
        }
    }};
}

#[macro_export]
macro_rules! filter_task {
    ($IN:literal, |$closure_param:ident: &$in:ty| $(-> bool)? $closure_block:block) => {{
        ::dist_iter::FilterTask {
            task: ::dist_iter::task!(
                |iter: UninitBuffer<$in, $IN>| -> impl IntoIterator<Item = $in, LEN = $IN> {
                    iter.filter(|$closure_param: &$in| $closure_block)
                }
            ),
        }
    }};
}

#[macro_export]
macro_rules! reduce_task {
    // TODO make sure in == in2 == in3
    ($IN:literal, |$closure_param1:ident: $in:ty, $closure_param2:ident $(:$in2:ty)?| $(-> $in3:ty)? $closure_block:block) => {{
        (
            ::dist_iter::ReduceTask {
                task: ::dist_iter::task!(
                    |iter: UninitBuffer<$in, $IN>| -> impl IntoIterator<Item = $in, LEN = 1> {
                        iter.reduce(|$closure_param1: $in, $closure_param2| $closure_block)
                    }
                ),
            },
            |$closure_param1: $in, $closure_param2| $closure_block,
        )
    }};
}
