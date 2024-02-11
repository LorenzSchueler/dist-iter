use mpi::{
    datatype::DynBufferMut, point_to_point::Message, topology::SimpleCommunicator, traits::*, Rank,
    Tag,
};

mod universe_guard;

use crate::universe_guard::UniverseGuard;

fn main() {
    let universe = UniverseGuard::new(mpi::initialize().unwrap());

    let world = universe.world();
    let size = world.size();
    let rank = world.rank();

    if rank == 0 {
        let work_queue = (100..120)
            .into_iter()
            .map(|t| Box::new(SquareTask::new(t)) as Box<dyn Task>)
            .chain(
                (0..10)
                    .into_iter()
                    .map(|t| Box::new(FibonacciTask::new(t)) as Box<dyn Task>),
            )
            .collect::<Vec<_>>();
        let mut send_idx = 0;
        let mut recv_idx = 0;

        for dest in 1..size {
            work_queue[send_idx].send(&world, dest);
            send_idx += 1;
        }

        while recv_idx < work_queue.len() {
            let (msg, status) = world.any_process().matched_probe();

            FUNCTIONS[status.tag() as usize].receive(msg);
            recv_idx += 1;

            if send_idx < work_queue.len() {
                work_queue[send_idx].send(&world, status.source_rank());
                send_idx += 1;
            }
        }
    } else {
        worker(&world);
    }
}

fn worker(world: &SimpleCommunicator) {
    loop {
        let (msg, status) = world.any_process().matched_probe();

        let function = FUNCTIONS[status.tag() as usize];
        let stop = function.execute(msg, world);
        if stop {
            break;
        }
    }
}

trait Function {
    fn execute(&self, msg: Message, world: &SimpleCommunicator) -> bool;

    fn receive(&self, msg: Message);

    fn tag(&self) -> Tag;
}

trait Task {
    fn send(&self, world: &SimpleCommunicator, dest: Rank);
}

struct Fibonacci {}

impl Function for Fibonacci {
    fn execute(&self, msg: Message, world: &SimpleCommunicator) -> bool {
        let (data, status) = msg.matched_receive();
        let result = fibonacci(data);
        world
            .process_at_rank(0)
            .send_with_tag(&result, status.tag());
        false
    }

    fn receive(&self, msg: Message) {
        let (data, status) = msg.matched_receive::<u64>();
        println!("root got data {:?} from {}", data, status.source_rank());
    }

    fn tag(&self) -> Tag {
        FIBONACCI_TAG
    }
}

struct FibonacciTask {
    data: u64,
}

impl FibonacciTask {
    fn new(data: u64) -> Self {
        Self { data }
    }
}

impl Task for FibonacciTask {
    fn send(&self, world: &SimpleCommunicator, dest: Rank) {
        world
            .process_at_rank(dest)
            .send_with_tag(&self.data, FIBONACCI_TAG);
    }
}

struct Square {}

impl Function for Square {
    fn execute(&self, msg: Message, world: &SimpleCommunicator) -> bool {
        let (data, status) = msg.matched_receive();
        let result = square(data);
        world
            .process_at_rank(0)
            .send_with_tag(&result, status.tag());
        false
    }

    fn receive(&self, msg: Message) {
        let (data, status) = msg.matched_receive::<i32>();
        println!("root got data {:?} from {}", data, status.source_rank());
    }

    fn tag(&self) -> Tag {
        SQUARE_TAG
    }
}

struct SquareTask {
    data: i32,
}

impl SquareTask {
    fn new(data: i32) -> Self {
        Self { data }
    }
}

impl Task for SquareTask {
    fn send(&self, world: &SimpleCommunicator, dest: Rank) {
        world
            .process_at_rank(dest)
            .send_with_tag(&self.data, SQUARE_TAG);
    }
}

struct End {}

impl Function for End {
    fn execute(&self, msg: Message, _world: &SimpleCommunicator) -> bool {
        let mut g: [u8; 0] = [];
        let mut buf = DynBufferMut::new(&mut g);
        let _ = msg.matched_receive_into(&mut buf);
        true
    }

    fn receive(&self, _msg: Message) {}

    fn tag(&self) -> Tag {
        END_TAG
    }
}

const END: End = End {};
const END_TAG: Tag = 0;
const FIBONACCI: Fibonacci = Fibonacci {};
const FIBONACCI_TAG: Tag = 1;
const SQUARE: Square = Square {};
const SQUARE_TAG: Tag = 2;
const FUNCTIONS: [&dyn Function; 3] = [&END, &FIBONACCI, &SQUARE]; // must be sorted and tags ascending without gaps

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
