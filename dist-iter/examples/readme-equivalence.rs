use mpi::traits::Equivalence;

#[derive(Equivalence)]
struct MyCustomType {
    x: i32,
    y: i32,
}

fn main() {}
