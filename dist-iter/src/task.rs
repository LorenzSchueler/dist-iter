use mpi::{traits::Equivalence, Tag};

#[doc(hidden)]
pub trait Task {
    type IN: Equivalence;
    type OUT: Equivalence;

    const TAG: Tag;
}

#[macro_export]
macro_rules! map_task {
    ($in:ty, $out:ty, $closure:expr) => {{
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
        ) -> bool {
            use ::dist_iter::mpi::point_to_point::{Destination, MatchedReceiveVec};

            let (data, status) = (msg, status).matched_receive_vec();
            let result = data.into_iter().map(function).collect::<Vec<_>>();
            process.send_with_tag(&result, status.tag());
            false
        }

        #[linkme::distributed_slice(::dist_iter::FUNCTION_REGISTRY)]
        static REGISTRY_ENTRY: ::dist_iter::RegistryEntry =
            (<ThisTask as ::dist_iter::Task>::TAG, execute);

        struct ThisTask {}

        impl ::dist_iter::Task for ThisTask {
            type IN = $in;
            type OUT = $out;

            const TAG: ::dist_iter::mpi::Tag = ::dist_iter::gen_tag(file!(), line!(), column!());
        }

        ThisTask {}
    }};
}
