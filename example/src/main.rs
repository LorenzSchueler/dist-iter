use dist_iter::{task, DistIterator, IntoDistIterator};

#[dist_iter::main]
fn main() {
    [1, 2, 3, 4, 5, 6, 7, 8, 9]
        .into_dist_iter()
        .map(task!(i32, u64, |x| (x * x) as u64))
        .map(|x| x * x)
        .into_dist_iter()
        .map(task!(u64, u32, |x| (x * x) as u32))
        .for_each(|v| println!("{v}"));
}
