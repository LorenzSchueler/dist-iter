use dist_iter::{map_task, DistIterator};

fn main() {
    [1i32, 2, 3, 4, 5]
        .into_iter()
        .dist_map(map_task!(|x: u32| -> u32 { x + 1 }));
}
