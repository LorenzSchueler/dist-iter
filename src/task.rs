use mpi::{
    point_to_point::Message,
    topology::{Process, SimpleCommunicator},
    traits::{Destination, Equivalence},
    Tag,
};

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

macro_rules! task {
    ($tag:expr, $in:ty, $out:ty, $closure:expr) => {
        |input: $in| {
            use mpi::{point_to_point::Message, topology::Process, Tag};

            /// used to make sure closure is of type `[fn($in) -> $out]`
            fn function(input: $in) -> $out {
                $closure(input)
            }

            fn execute(msg: Message, process: Process<'_, SimpleCommunicator>) -> bool {
                let (data, status) = msg.matched_receive();
                let result = function(data);
                process.send_with_tag(&result, status.tag());
                false
            }

            #[linkme::distributed_slice(crate::function_registry::FUNCTION_REGISTRY)]
            static REGISTRY_ENTRY: (Tag, fn(Message, Process<'_, SimpleCommunicator>) -> bool) =
                ($tag, execute); // tag must be generated uniquely

            struct ThisTask {
                data: $in,
            }

            impl ThisTask {
                pub fn new(data: $in) -> Self {
                    Self { data }
                }
            }

            impl crate::task::Task for ThisTask {
                type IN = $in;
                type OUT = $out;

                const TAG: Tag = $tag;

                fn get_data(&self) -> &Self::IN {
                    &self.data
                }
            }

            ThisTask::new(input)
        }
    };
}

pub(crate) use task;
