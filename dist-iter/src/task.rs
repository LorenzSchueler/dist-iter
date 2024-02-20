use mpi::{traits::Equivalence, Tag};

#[doc(hidden)]
pub trait MapTask<const N: usize> {
    type In: Equivalence;
    type Out: Equivalence;

    const TAG: Tag;
}

#[doc(hidden)]
pub trait MapChunkTask<const N: usize> {
    type In: Equivalence;
    type Out: Equivalence;

    const TAG: Tag;
}

#[doc(hidden)]
pub trait FilterTask<const N: usize> {
    type Item: Equivalence;

    const TAG: Tag;
}

#[doc(hidden)]
pub trait ReduceTask<const N: usize> {
    type Item: Equivalence;

    const TAG: Tag;
}

#[macro_export]
macro_rules! map_task {
    ($n: expr, $in:ty, $out:ty, $closure:expr) => {{
        // make sure closure is of type `[fn($in) -> $out]`
        fn function(input: $in) -> $out {
            $closure(input)
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
            let mut send_buf = ::dist_iter::UninitBuffer::<_, $n>::new();
            for item in recv_buf {
                send_buf.push_back_unchecked(function(item));
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
            (<ThisTask as ::dist_iter::MapTask<$n>>::TAG, execute);

        struct ThisTask {}

        impl ::dist_iter::MapTask<$n> for ThisTask {
            type In = $in;
            type Out = $out;

            const TAG: ::dist_iter::mpi::Tag = ::dist_iter::gen_tag(file!(), line!(), column!());
        }

        ThisTask {}
    }};
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
            (<ThisTask as ::dist_iter::MapChunkTask<$n>>::TAG, execute);

        struct ThisTask {}

        impl ::dist_iter::MapChunkTask<$n> for ThisTask {
            type In = $in;
            type Out = $out;

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
            (<ThisTask as ::dist_iter::MapChunkTask<$n>>::TAG, execute);

        struct ThisTask {}

        impl ::dist_iter::MapChunkTask<$n> for ThisTask {
            type In = $in;
            type Out = $in;

            const TAG: ::dist_iter::mpi::Tag = ::dist_iter::gen_tag(file!(), line!(), column!());
        }

        ThisTask {}
    }};
}

#[macro_export]
macro_rules! filter_task {
    ($n: expr, $item:ty, $closure:expr) => {{
        // make sure closure is of type `[fn($item) -> bool]`
        fn function(input: &$item) -> bool {
            $closure(input)
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
            let mut send_buf = ::dist_iter::UninitBuffer::<_, $n>::new();
            for item in recv_buf {
                if function(&item) {
                    send_buf.push_back_unchecked(item);
                }
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
            (<ThisTask as ::dist_iter::FilterTask<$n>>::TAG, execute);

        struct ThisTask {}

        impl ::dist_iter::FilterTask<$n> for ThisTask {
            type Item = $item;

            const TAG: ::dist_iter::mpi::Tag = ::dist_iter::gen_tag(file!(), line!(), column!());
        }

        ThisTask {}
    }};
}

#[macro_export]
macro_rules! reduce_task {
    ($n: expr, $item:ty, $closure:expr) => {{
        // make sure closure is of type `[fn($item, $item) -> $item]`
        fn function(acc: $item, item: $item) -> $item {
            $closure(acc, item)
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
            let result = recv_buf.reduce(function).unwrap(); // will not panic because recv_buf will never be empty
            eprintln!("    < [{}] reduce result", std::process::id());
            process.send_with_tag(&result, status.tag());
            ::dist_iter::WorkerMode::Continue
        }

        #[linkme::distributed_slice(::dist_iter::FUNCTION_REGISTRY)]
        static REGISTRY_ENTRY: ::dist_iter::RegistryEntry =
            (<ThisTask as ::dist_iter::ReduceTask<$n>>::TAG, execute);

        struct ThisTask {}

        impl ::dist_iter::ReduceTask<$n> for ThisTask {
            type Item = $item;

            const TAG: ::dist_iter::mpi::Tag = ::dist_iter::gen_tag(file!(), line!(), column!());
        }

        (ThisTask {}, $closure)
    }};
}
