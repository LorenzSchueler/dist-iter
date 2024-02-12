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

            #[linkme::distributed_slice(crate::dispatch::FUNCTION_REGISTRY)]
            static REGISTRY_ENTRY: (
                Tag,
                fn(Message, Process<'_, SimpleCommunicator>) -> bool,
                fn(Message) -> Box<dyn std::any::Any>,
            ) = ($tag, execute, crate::traits::receive::<$out>); // tag must be generated uniquely

            struct ThisTask {
                data: $in,
            }

            impl ThisTask {
                pub fn new(data: $in) -> Self {
                    Self { data }
                }
            }

            impl crate::traits::Task for ThisTask {
                fn send(&self, process: Process<'_, SimpleCommunicator>) {
                    process.send_with_tag(&self.data, REGISTRY_ENTRY.0);
                }
            }

            ThisTask::new(input)
        }
    };
}

pub(crate) use task;
