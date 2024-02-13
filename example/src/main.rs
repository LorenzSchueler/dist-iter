use dist_iter::{task, IntoDistIter};

#[dist_iter::main]
fn main() {
    (0..10)
        .map(task!(1, i32, i32, |x| x * x))
        .into_dist_iter()
        .for_each(|v| println!("{v}"));
    (0..10)
        .map(task!(2, u8, u8, |x| x * 2))
        .into_dist_iter()
        .for_each(|v| println!("{v}"));
}
