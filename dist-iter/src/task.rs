use mpi::{traits::Equivalence, Tag};

#[doc(hidden)]
pub trait MapChunkTask {
    type In: Equivalence;
    type Out: Equivalence;

    const N: usize;
    const TAG: Tag;
}

#[macro_export]
macro_rules! map_chunk_task {
    (|$closure_param:ident: UninitBuffer<$in:ty, $n:literal>| -> impl IntoIterator<Item = $out:ty> $closure_block:block) => {{
        fn function(
            $closure_param: ::dist_iter::UninitBuffer<$in, $n>,
        ) -> impl IntoIterator<Item = $out> {
            $closure_block
        }

        fn execute(
            msg: ::dist_iter::mpi::point_to_point::Message,
            status: ::dist_iter::mpi::point_to_point::Status,
            process: ::dist_iter::mpi::topology::Process<
                '_,
                ::dist_iter::mpi::topology::SimpleCommunicator,
            >,
        ) -> ::dist_iter::WorkerMode {
            use ::dist_iter::mpi::point_to_point::Destination;

            let (recv_buf, tag) = ::dist_iter::UninitBuffer::<_, $n>::from_matched_receive(msg);
            eprintln!(
                "    > [{}] data of length {:?}",
                std::process::id(),
                recv_buf.len()
            );
            let result = function(recv_buf);

            let mut send_buf = ::dist_iter::UninitBuffer::<_, $n>::new();
            for item in result {
                send_buf.push_back_unchecked(item);
            }
            eprintln!(
                "    < [{}] data of length {:?}",
                std::process::id(),
                send_buf.len()
            );
            process.send_with_tag(&*send_buf, status.tag());
            ::dist_iter::WorkerMode::Continue
        }

        #[linkme::distributed_slice(::dist_iter::FUNCTION_REGISTRY)]
        static REGISTRY_ENTRY: ::dist_iter::RegistryEntry =
            (<ThisTask as ::dist_iter::MapChunkTask>::TAG, execute);

        struct ThisTask {}

        impl ::dist_iter::MapChunkTask for ThisTask {
            type In = $in;
            type Out = $out;

            const N: usize = $n;
            const TAG: ::dist_iter::mpi::Tag = ::dist_iter::gen_tag(file!(), line!(), column!());
        }

        ThisTask {}
    }};
    (|$closure_param:ident: &mut UninitBuffer<$in:ty, $n:literal>| $closure_block:block) => {{
        fn function($closure_param: &mut ::dist_iter::UninitBuffer<$in, $n>) {
            $closure_block
        }

        fn execute(
            msg: ::dist_iter::mpi::point_to_point::Message,
            status: ::dist_iter::mpi::point_to_point::Status,
            process: ::dist_iter::mpi::topology::Process<
                '_,
                ::dist_iter::mpi::topology::SimpleCommunicator,
            >,
        ) -> ::dist_iter::WorkerMode {
            use ::dist_iter::mpi::point_to_point::Destination;

            let (mut buf, tag) = ::dist_iter::UninitBuffer::<_, $n>::from_matched_receive(msg);
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
            process.send_with_tag(&*buf, status.tag());
            ::dist_iter::WorkerMode::Continue
        }

        #[linkme::distributed_slice(::dist_iter::FUNCTION_REGISTRY)]
        static REGISTRY_ENTRY: ::dist_iter::RegistryEntry =
            (<ThisTask as ::dist_iter::MapChunkTask>::TAG, execute);

        struct ThisTask {}

        impl ::dist_iter::MapChunkTask for ThisTask {
            type In = $in;
            type Out = $in;

            const N: usize = $n;
            const TAG: ::dist_iter::mpi::Tag = ::dist_iter::gen_tag(file!(), line!(), column!());
        }

        ThisTask {}
    }};
}

#[macro_export]
macro_rules! map_task {
    ($n:literal, |$closure_param:ident: $in:ty| -> $out:ty $closure_block:block) => {{
        ::dist_iter::map_chunk_task!(
            |iter: UninitBuffer<$in, $n>| -> impl IntoIterator<Item = $out> {
                iter.map(|$closure_param: $in| $closure_block)
            }
        )
    }};
}

#[macro_export]
macro_rules! filter_task {
    ($n:literal, |$closure_param:ident: &$in:ty| $(-> bool)? $closure_block:block) => {{
        ::dist_iter::map_chunk_task!(
            |iter: UninitBuffer<$in, $n>| -> impl IntoIterator<Item = $in> {
                iter.filter(|$closure_param: &$in| $closure_block)
            }
        )
    }};
}

#[macro_export]
macro_rules! reduce_task {
    // TODO make sure in == in2 == in3
    ($n:literal, |$closure_param1:ident: $in:ty, $closure_param2:ident $(:$in2:ty)?| $(-> $in3:ty)? $closure_block:block) => {{
        (
            ::dist_iter::map_chunk_task!(
                |iter: UninitBuffer<$in, $n>| -> impl IntoIterator<Item = $in> {
                    iter.reduce(|$closure_param1: $in, $closure_param2| $closure_block)
                }
            ),
            |$closure_param1: $in, $closure_param2| $closure_block,
        )
    }};
}
