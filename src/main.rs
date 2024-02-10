use mpi::traits::*;
use std::io::Write;
#[derive(Clone, Debug, Equivalence)]
struct Wrapper {
    function: [u8; 20],
    input: u64,
}

fn main() {
    let universe = mpi::initialize().unwrap();
    let world = universe.world();
    let size = world.size();
    let rank = world.rank();

    let mut function = [0u8; 20];
    function.as_mut_slice().write(FIBONACCI.as_bytes()).unwrap();

    let msg = (40..(40 + size - 1) as u64)
        .into_iter()
        .map(|x| Wrapper { function, input: x })
        .collect::<Vec<_>>();
    if rank == 0 {
        mpi::request::scope(|scope| {
            let mut send_requests = vec![];
            for dest in 1..size {
                send_requests.push(
                    world
                        .process_at_rank(dest)
                        .immediate_send(scope, &msg[dest as usize - 1]),
                )
            }
            for req in send_requests {
                req.wait_without_status();
            }
        });
        let mut results = vec![0; size as usize - 1];
        for source in 1..size {
            let (msg, status) = world.process_at_rank(source).receive::<u64>();
            println!("root got message {:?} from {}", msg, status.source_rank());
            results[status.source_rank() as usize - 1] = msg;
        }
    } else {
        let (msg, _) = world.any_process().receive::<Wrapper>();

        println!("Process {} got message {:?}", rank, msg);

        let result = [dispatch(msg)];
        world.process_at_rank(0).send(&result);
    }
}

const FIBONACCI: &str = "fibonacci";
const SQUARE: &str = "square";
fn dispatch(wrapper: Wrapper) -> u64 {
    let name = std::str::from_utf8(&wrapper.function).unwrap();
    let name = &name[0..name.find('\0').unwrap()];
    println!("{name:?} = {FIBONACCI:?}");
    match name {
        FIBONACCI => fibonacci(wrapper.input),
        SQUARE => square(wrapper.input),
        _ => panic!("unknown function {name:?}"),
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

fn square(n: u64) -> u64 {
    n * n
}
