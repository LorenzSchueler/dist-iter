use dist_iter::{task, IntoDistIter};

#[dist_iter::main]
fn main() {
    (0..10)
        .map(task!(2, i32, i32, |x| x * x))
        .into_dist_iter(world)
        .for_each(|v| println!("{v}"));
    (0..10)
        .map(task!(1, u8, u8, |x| x * 2))
        .into_dist_iter(world)
        .for_each(|v| println!("{v}"));
}
