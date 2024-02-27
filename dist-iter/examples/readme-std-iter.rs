fn main() {
    // ...
    let _ = [1, 2, 3, 4, 5]
        .into_iter()
        // possibly more adapters
        .map(|x: i32| -> i32 {
            // very expensive operation
            x * x
        })
        // possibly more adapters
        .collect::<Vec<_>>();
    // ...
}
