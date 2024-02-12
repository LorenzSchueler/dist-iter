use mpi::{
    point_to_point::Message,
    topology::{Process, SimpleCommunicator},
    traits::{Destination, Equivalence},
    Tag,
};

#[doc(hidden)]
pub trait Task {
    type IN: Equivalence;
    type OUT: Equivalence;

    const TAG: Tag;

    fn get_data(&self) -> &Self::IN;

    fn send(&self, process: Process<'_, SimpleCommunicator>) {
        process.send_with_tag(self.get_data(), Self::TAG);
    }

    fn receive<T: 'static + Equivalence>(msg: Message) -> T {
        let (data, _) = msg.matched_receive::<T>();
        data
    }
}

#[macro_export]
macro_rules! task {
    ($tag:expr, $in:ty, $out:ty, $closure:expr) => {
        |input: $in| {
            /// used to make sure closure is of type `[fn($in) -> $out]`
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

            #[linkme::distributed_slice(::dist_iter::FUNCTION_REGISTRY)]
            static REGISTRY_ENTRY: ::dist_iter::RegistryEntry = ($tag, execute); // tag must be generated uniquely

            struct ThisTask {
                data: $in,
            }

            impl ThisTask {
                pub fn new(data: $in) -> Self {
                    Self { data }
                }
            }

            impl ::dist_iter::Task for ThisTask {
                type IN = $in;
                type OUT = $out;

                const TAG: ::dist_iter::mpi::Tag = $tag;

                fn get_data(&self) -> &Self::IN {
                    &self.data
                }
            }

            ThisTask::new(input)
        }
    };
}
