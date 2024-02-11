use std::ops::Deref;

use mpi::{
    datatype::DynBufferMut, environment::Universe, point_to_point::Message,
    topology::SimpleCommunicator, traits::*, Tag,
};

struct UniverseGuard {
    comm: Universe,
}

impl Drop for UniverseGuard {
    fn drop(&mut self) {
        let world = self.world();
        if world.rank() == 0 {
            let size = world.size();
            let function = END;
            let data: [u8; 0] = [];
            mpi::request::scope(|scope| {
                let mut send_requests = vec![];
                println!("sending end");
                for dest in 1..size {
                    send_requests.push(
                        world
                            .process_at_rank(dest)
                            .immediate_send_with_tag(scope, &data, function),
                    );
                }
                for req in send_requests {
                    req.wait_without_status();
                }
                println!("done");
            });
        }
    }
}

impl Deref for UniverseGuard {
    type Target = Universe;

    fn deref(&self) -> &Self::Target {
        &self.comm
    }
}

fn main() {
    let universe = UniverseGuard {
        comm: mpi::initialize().unwrap(),
    };
    let world = universe.world();
    let size = world.size();
    let rank = world.rank();

    if rank == 0 {
        let function = FIBONACCI;
        let data = (30..(30 + size - 1) as u64).into_iter().collect::<Vec<_>>();
        mpi::request::scope(|scope| {
            let mut send_requests = vec![];
            println!("sending first tasks");
            for dest in 1..size {
                send_requests.push(world.process_at_rank(dest).immediate_send_with_tag(
                    scope,
                    &data[dest as usize - 1],
                    function,
                ))
            }
            for req in send_requests {
                req.wait_without_status();
            }
            println!("done");
        });
        let mut results = vec![0; size as usize - 1];
        for source in 1..size {
            let (msg, status) = world.process_at_rank(source).receive::<u64>();
            println!("root got message {:?} from {}", msg, status.source_rank());
            results[status.source_rank() as usize - 1] = msg;
        }

        let function = SQUARE;
        let data = (30..(30 + size - 1) as i32).into_iter().collect::<Vec<_>>();
        mpi::request::scope(|scope| {
            let mut send_requests = vec![];
            println!("sending second tasks");
            for dest in 1..size {
                send_requests.push(world.process_at_rank(dest).immediate_send_with_tag(
                    scope,
                    &data[dest as usize - 1],
                    function,
                ))
            }
            for req in send_requests {
                req.wait_without_status();
            }
            println!("done");
        });
        let mut results = vec![0; size as usize - 1];
        for source in 1..size {
            let (msg, status) = world.process_at_rank(source).receive::<i32>();
            println!("root got message {:?} from {}", msg, status.source_rank());
            results[status.source_rank() as usize - 1] = msg;
        }
    } else {
        worker(&world);
    }
}

fn worker(world: &SimpleCommunicator) {
    loop {
        let (msg, status) = world.any_process().matched_probe();

        if dispatch(msg, status.tag(), &world) {
            break;
        }
    }
}

const END: Tag = 0;
const FIBONACCI: Tag = 1;
const SQUARE: Tag = 2;
fn dispatch(msg: Message, tag: Tag, world: &SimpleCommunicator) -> bool {
    match tag {
        END => {
            let mut g: [u8; 0] = [];
            let mut buf = DynBufferMut::new(&mut g);
            let _ = msg.matched_receive_into(&mut buf);
            true
        }
        FIBONACCI => {
            let (data, _) = msg.matched_receive();
            let result = fibonacci(data);
            world.process_at_rank(0).send(&result);
            false
        }
        SQUARE => {
            let (data, _) = msg.matched_receive();
            let result = square(data);
            world.process_at_rank(0).send(&result);
            false
        }
        _ => panic!("unknown function tag {tag:?}"),
    }
}

fn fibonacci(n: u64) -> u64 {
    if n == 0 {
        return 0;
    } else if n == 1 {
        return 1;
    } else {
        return fibonacci(n - 1) + fibonacci(n - 2);
    }
}

fn square(n: i32) -> i32 {
    n * n
}
