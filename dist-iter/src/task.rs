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
    ($in:ty, $out:ty, $closure:expr) => {
        |input: $in| {
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

            #[linkme::distributed_slice(::dist_iter::FUNCTION_REGISTRY)]
            static REGISTRY_ENTRY: ::dist_iter::RegistryEntry =
                (<ThisTask as ::dist_iter::Task>::TAG, execute);

            struct ThisTask {
                data: $in,
            }

            impl ThisTask {
                pub fn new(data: $in) -> Self {
                    Self { data }
                }
            }

            // try to generate unique tag
            // uniqueness is not guaranteed at compile time but check at runtime
            const FILE_HASH: [u8; 20] = const_sha1::sha1(file!().as_bytes()).as_bytes();
            const TAG: u32 = ((line!() * 2_u32.pow(16)) + column!())
                ^ (((FILE_HASH[0] as u32) << 24)
                    + ((FILE_HASH[1] as u32) << 16)
                    + ((FILE_HASH[2] as u32) << 8)
                    + ((FILE_HASH[3] as u32) << 0))
                ^ (((FILE_HASH[4] as u32) << 24)
                    + ((FILE_HASH[5] as u32) << 16)
                    + ((FILE_HASH[6] as u32) << 8)
                    + ((FILE_HASH[7] as u32) << 0))
                ^ (((FILE_HASH[8] as u32) << 24)
                    + ((FILE_HASH[9] as u32) << 16)
                    + ((FILE_HASH[10] as u32) << 8)
                    + ((FILE_HASH[11] as u32) << 0))
                ^ (((FILE_HASH[12] as u32) << 24)
                    + ((FILE_HASH[13] as u32) << 16)
                    + ((FILE_HASH[14] as u32) << 8)
                    + ((FILE_HASH[15] as u32) << 0));

            impl ::dist_iter::Task for ThisTask {
                type IN = $in;
                type OUT = $out;

                const TAG: ::dist_iter::mpi::Tag = (TAG as i32).abs();

                fn get_data(&self) -> &Self::IN {
                    &self.data
                }
            }

            ThisTask::new(input)
        }
    };
}
