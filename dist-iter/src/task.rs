use mpi::{traits::Equivalence, Tag};

#[doc(hidden)]
pub trait MapTask<const N: usize> {
    type In: Equivalence;
    type Out: Equivalence;

    const TAG: Tag;
}

#[doc(hidden)]
pub trait MapIterTask<const N: usize> {
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

            let mut recv_buf = ::dist_iter::UninitBuffer::<_, $n>::new();
            recv_buf.matched_receive_into(msg);
            eprintln!(
                "    > [{}] data of length {:?}",
                std::process::id(),
                recv_buf.init_count()
            );
            let mut send_buf = ::dist_iter::UninitBuffer::<_, $n>::new();
            for item in recv_buf {
                send_buf.push_back_unchecked(function(item));
            }
            eprintln!(
                "    < [{}] data of length {:?}",
                std::process::id(),
                send_buf.init_count()
            );
            process.send_with_tag(send_buf.init_slice(), status.tag());
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
macro_rules! map_iter_task {
    ($n: expr, $in:ty, $out:ty, $closure:expr) => {{
        // make sure closure is of type `[fn(::dist_iter::UninitBuffer<$in, $n>) -> impl IntoIterator<Item = $out>]`
        fn function(input: ::dist_iter::UninitBuffer<$in, $n>) -> impl IntoIterator<Item = $out> {
            let c: fn(::dist_iter::UninitBuffer<$in, $n>) -> _ = $closure;
            c(input)
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

            let mut recv_buf = ::dist_iter::UninitBuffer::<_, $n>::new();
            recv_buf.matched_receive_into(msg);
            eprintln!(
                "    > [{}] data of length {:?}",
                std::process::id(),
                recv_buf.init_count()
            );
            let result = function(recv_buf);

            let mut send_buf = ::dist_iter::UninitBuffer::<_, $n>::new();
            for item in result {
                send_buf.push_back_unchecked(item);
            }
            eprintln!(
                "    < [{}] data of length {:?}",
                std::process::id(),
                send_buf.init_count()
            );
            process.send_with_tag(send_buf.init_slice(), status.tag());
            ::dist_iter::WorkerMode::Continue
        }

        #[linkme::distributed_slice(::dist_iter::FUNCTION_REGISTRY)]
        static REGISTRY_ENTRY: ::dist_iter::RegistryEntry =
            (<ThisTask as ::dist_iter::MapIterTask<$n>>::TAG, execute);

        struct ThisTask {}

        impl ::dist_iter::MapIterTask<$n> for ThisTask {
            type In = $in;
            type Out = $out;

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

            let mut recv_buf = ::dist_iter::UninitBuffer::<_, $n>::new();
            recv_buf.matched_receive_into(msg);
            eprintln!(
                "    > [{}] data of length {:?}",
                std::process::id(),
                recv_buf.init_count()
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
                send_buf.init_count()
            );
            process.send_with_tag(send_buf.init_slice(), status.tag());
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

            let mut recv_buf = ::dist_iter::UninitBuffer::<_, $n>::new();
            recv_buf.matched_receive_into(msg);
            eprintln!(
                "    > [{}] data of length {:?}",
                std::process::id(),
                recv_buf.init_count()
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
