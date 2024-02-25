# `dist-iter` - Parallel Iterator Adapters for Distributed Memory Systems

## Overview
`dist-iter` provides iterator adapters which look similar to those in `std::iter::Iterator` but process data in parallel.
Internally they use MPI for message passing between the master and worker ranks.
The master rank sends out chunks of items from the underlying iterator to worker ranks to process, and provides the results again as an iterator.

Example:
- normal iterator adapters from `std::iter::Iterator`
    ```rs
    fn main() {
        // ...
        let results: Vec<_> = [1, 2, 3, 4, 5]
            .into_iter()
            // possibly more adapters
            .map(|x: i32| -> i32 { 
                // very expensive operation
                x * x 
            })
            // possibly more adapters
            .collect();
        // ...
    }
    ```
- parallelized map
    ```rs
    use dist_iter::{map_task, DistIterator};

    #[dist_iter::main]
    fn main() {
        // ...
        let results: Vec<_> = [1, 2, 3, 4, 5]
            .into_iter()
            // possibly more adapters
            .dist_map(map_task!(2, |x: i32| -> i32 {
                // very expensive operation
                x * x 
            }))
            // possibly more adapters
            .collect();
        // ...
    }
    ```

## How do I use this library?

1. Annotate your `main` function with `#[dist_iter::main]`
2. replace those adapters which should be executed in parallel with their `dist_*` equivalent and wrap the closure with the appropriate macro
3. make sure only a single thread calls methods from `DistIterator`

## How does this work internally?

`dist_iter::DistIterator` is an extension trait on `std::iter::Iterator`.
It provides methods that are inspired by those in `std::iter::Iterator` but are executed in parallel.
Each method returns a type which itself implements `std::iter::Iterator` or in some cases (e.g. reduce) the final result.

Multiple items from the underlying iterator are collected into a chunk (for now `[MaybeUninit<T>]`; no allocation required) and then send to a worker rank.
On the first call to `next()` *every* worker rank will be sent a chunk (this also means that `next()` will be called many times on the underlying iterator).
After that the master rank blocks until it receives the first response.
Upon receiving the data, the worker rank executes the closure and sends back the result.
It then blocks until it receives the next chunk.
When the master rank receives a response, it stores the response and sends out a new chunk (if there is more work to do).
The stored response is then used to fulfill the following calls to `next()` until all items from this chunk have been returned.
After that the master rank again blocks until it receives the next chunk.

Implications:
- elements can (and likely will) be reordered
- if the next adapter makes no progress the dist adapter will also make no progress once all worker ranks have finished their current work and wait for the send call to return
- work imbalance is no issue because if a rank takes longer to finish chunks it will just process fewer

The `main` function has to be annotated with `#[dist_iter::main]`.
This macro essentially moves the code from `main` into a `master` function.
It then creates a new `main` function which initializes MPI and checks if the current rank is the master rank (rank 0).
The master rank then executes the new `master` function while all other ranks execute a `worker` function.
This `worker` function has an infinite loop.
In this loop it first issues a *matching probe*, then looks up the function to execute, executes this function and finally sends back the result.

In order to tell the workers what datatype the chunk has and which function to execute, a function registry is used.
This registry associates MPI Tags with function pointers.
Those functions are responsible for receiving and processing the data and sending back the response. 
In the infinite loop of the workers a *matching probe* is used to get the tag of the next message.
This tag is then used to look up the function in the registry. 
This function is the called with the `MPI_Message` so that it call actually receive and process the data.

The function registry is build at link time.
In order to register a function a `*_task!(...)` macro has to be used.
There is one macro for each method in `DistIterator` (e.g. for `dist_map` there is `map_task!`, for `dist_filter` there is `filter_task!`).
The macros take the closure (which is cast to a function pointer) with type annotations and also the chunk sizes for input (what the worker receives) and output (what the worker responds with).

## Provided methods in `DistIterator`

- `dist_map`: maps one item
    ```rs
    // .dist_map(map_task!(<chunk size>, |x: <input type>| -> <output type> { ... }))
    my_iter.dist_map(map_task!(10, |x: i32| -> i32 { x * x }))
    ```
- `dist_filter`
    ```rs
    // .dist_filter(filter_task!(<chunk size>, |x: &<type>| { ... }))
    // OR
    // .dist_filter(filter_task!(<chunk size>, |x: &<type>| -> bool { ... }))
    my_iter.dist_filter(filter_task!(10, |x: &i32| { x % 2 == 0 }))
    ```
- `dist_reduce`
    ```rs
    // .dist_reduce(reduce_task!(<chunk size>, |x: <type>, y| { ... }));
    // OR
    // .dist_reduce(reduce_task!(<chunk size>, |x: <type>, y: <type>| { ... }));
    // OR
    // .dist_reduce(reduce_task!(<chunk size>, |x: <type>, y: <type>| -> <type> { ... }));
    // NOTE: type must be the same for x, y and return value
    my_iter.dist_reduce(reduce_task!(10, |x: i32, y| { x + y }));
    ```
- `dist_map_chunk`
    ```rs
    // .dist_map_chunk(map_chunk_task!(
    //     |iter: UninitBuffer<<input type>, <input chunk size>>| 
    //     -> impl IntoIterator<Item = <output type>, LEN = <output chunk size>> {
    //         ...
    //     }
    // ))
    // OR
    // .dist_map_chunk(map_chunk_task!(|buf: &mut UninitBuffer<<type>, <chunk size>>| {
    //     ...
    // }))
    my_iter.dist_map_chunk(map_chunk_task!(
        |iter: UninitBuffer<i32, 5>| -> impl IntoIterator<Item = i32, LEN = 10> {
            // return IntoIterator which will return at most LEN items
            iter.map(|x| x * x).filter(|x| x % 2 == 0)
        }
    ))
    my_iter.dist_map_chunk(map_chunk_task!(|buf: &mut UninitBuffer<i32, 10>| {
        // modify in place
        for item in buf.deref_mut() {
            *item += 1;
        }
    }))
    ```

## Multiple Parallel Adapters

Since the return value of the `dist_*` methods are Iterators themselves, multiple parallel adapters can be applied after each other.

E.g.
```rs
my_iter
    .dist_map(map_task!(10, |x: i32| -> i32 { x * x }))
    .dist_filter(filter_task!(10, |x: &i32| { x % 2 == 0 }))
```

However, keep in mind that in this case the results of `dist_map` are sent back to the master rank and then the received results are sent out again to worker ranks as input for `dist_filter`.

If this is not intended, and you want to apply both closures at the worker before sending back the result `dist_map_chunk` can be used.
```rs
my_iter.dist_map_chunk(map_chunk_task!(
    |iter: UninitBuffer<i32, 5>| -> impl IntoIterator<Item = i32, LEN = 5> {
        iter.map(|x| x * x).filter(|x| x % 2 == 0)
    }
))
```

## Chunk Sizes

Chunk sizes can greatly influence how performant the program will be.
The larger the chunks, the fewer message transfers are necessary.
However, big (and few) chunks make work balancing worse if the processing time is data dependent.

Chunk sizes have to be known at compile time, in order to be able to avoid allocations. 
The master rank will always fill the chunks completely, unless the underlying iterator is finished.

On the worker side it depends on the kind of adapter:
- Some adapters (like `dist_map`) do not modify the number of items. 
Therefore, the chunks which get returned will also be completely filled.
- Others (like `dist_filter`) modify the number of items. 
Therefore, the response may contain fewer items.
- Adapters like `dist_reduce` return only a single value.
- `dist_map_chunk` can be used to implement every other adapter.
The number of returned items can be lower or higher than the number of received items.
Therefore, the input and output chunk size can be specified separately.

## Install

```sh
sudo apt install libopenmpi-dev
cargo install cargo-mpirun
```

NOTE: additionally to depending on `dist_iter` you also have to add `linkme` to your dependencies.
