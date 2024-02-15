use mpi::{traits::Equivalence, Tag};

#[doc(hidden)]
pub trait Task {
    type IN: Equivalence;
    type OUT: Equivalence;

    const TAG: Tag;
}

#[macro_export]
macro_rules! task {
    ($in:ty, $out:ty, $closure:expr) => {{
        // make sure closure is of type `[fn($in) -> $out]`
        fn function(input: $in) -> $out {
            $closure(input)
        }

        fn execute(
            msg: ::dist_iter::mpi::point_to_point::Message,
            process: ::dist_iter::mpi::topology::Process<
                '_,
                ::dist_iter::mpi::topology::SimpleCommunicator,
            >,
        ) -> bool {
            use ::dist_iter::mpi::point_to_point::Destination;

            let (data, status) = msg.matched_receive();
            let result = function(data);
            process.send_with_tag(&result, status.tag());
            false
        }

        const TAG: ::dist_iter::mpi::Tag = ::dist_iter::gen_tag(file!(), line!(), column!());
        #[linkme::distributed_slice(::dist_iter::FUNCTION_REGISTRY)]
        static REGISTRY_ENTRY: ::dist_iter::RegistryEntry = (TAG, execute);

        struct ThisTask {}

        impl ::dist_iter::Task for ThisTask {
            type IN = $in;
            type OUT = $out;

            const TAG: ::dist_iter::mpi::Tag = TAG;
        }

        ThisTask {}
    }};
}
