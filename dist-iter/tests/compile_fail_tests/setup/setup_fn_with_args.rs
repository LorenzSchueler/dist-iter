use dist_iter::{for_each_task, DistIterator};

#[dist_iter::main(setup = setup)]
fn main() {
    [1, 2, 3, 4]
        .into_iter()
        .dist_for_each(for_each_task!(CHUNK_SIZE = 2, |x: i32| {
            println!("{x}");
        }));
}

fn setup(_arg: bool) {}
