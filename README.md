# `dist-iter` - Parallel Iterator Adapters for Distributed Memory Systems

## Overview
`dist-iter` provides iterator adapters which look similar to those in `std::iter::Iterator` but process data in parallel.
Internally they use MPI for message passing between the master and worker ranks.
The master rank sends out chunks of items from the underlying iterator to worker ranks to process, and provides the results again as an iterator.

Example:
- normal iterator adapters from `std::iter::Iterator`
    https://github.com/LorenzSchueler/dist-iter/blob/885638d42b080fc7033797be07ee9cc0a1c2fea7/dist-iter/examples/readme-std-iter.rs#L1-L13
- parallelized map
    https://github.com/LorenzSchueler/dist-iter/blob/885638d42b080fc7033797be07ee9cc0a1c2fea7/dist-iter/examples/readme-dist-iter.rs#L1-L16

## How to use this library

1. Annotate your `main` function with `#[dist_iter::main]`.
   In case some setup code should be run on every rank, annotate with `#[dist_iter::main(setup = my_setup_fn)]` instead.
   This is useful for things like initializing `tracing_subscriber`.
   This setup function is executed before the code in main.
2. Replace those adapters which should be executed in parallel with their `dist_*` equivalent and wrap the closure with the appropriate macro
3. The items which are sent must implement `mpi::traits::Equivalence`. `Equivalence` is already implemented for all integer and floating point types and for bool. You can derive `Equivalence` for your own structs if all fields implement `Equivalence`.
    https://github.com/LorenzSchueler/dist-iter/blob/885638d42b080fc7033797be07ee9cc0a1c2fea7/dist-iter/examples/readme-equivalence.rs#L3-L7

## How it works

`dist_iter::DistIterator` is an extension trait on `std::iter::Iterator`.
It provides methods that are inspired by those in `std::iter::Iterator` but are executed in parallel.
Each method returns a type which itself implements `std::iter::Iterator` or in some cases (e.g. reduce) the final result.

Multiple items from the underlying iterator are collected into a chunk (for now `[MaybeUninit<T>]`; no allocation required) and then send to a worker rank.
On the first call to `next()` *every* worker rank will be sent a chunk (this also means that `next()` will be called multiple times on the underlying iterator).
After that the master rank blocks until it receives the first response.
Upon receiving the data, the worker rank executes the closure and sends back the result.
It then blocks until it receives the next chunk.
When the master rank receives a response, it stores the response and sends out a new chunk (if there is more work to do).
The stored response is then used to fulfill the following calls to `next()` until all items from this chunk have been returned.
After that the master rank again blocks until it receives the next chunk.

Implications:
- elements can (and likely will) be reordered
- if the next adapter makes no progress the dist adapter will also make no progress once all worker ranks have finished their current work and wait for the send call to return
- work imbalance is no issue because if a rank takes longer to finish chunks, it will just process fewer of them

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
This function is then called with the `MPI_Message` so that it call actually receive and process the data.

The function registry is build at link time.
In order to register a function a `*_task!(...)` macro has to be used.
There is one macro for each method in `DistIterator` (e.g. for `dist_map` there is `map_task!`, for `dist_filter` there is `filter_task!`).
The macros take the closure (which is cast to a function pointer) with type annotations and also the chunk sizes for input (what the worker receives) and output (what the worker responds with).

## Provided methods in `DistIterator`

- `dist_map`
    https://github.com/LorenzSchueler/dist-iter/blob/885638d42b080fc7033797be07ee9cc0a1c2fea7/dist-iter/examples/readme-provided-methods.rs#L6-L7
- `dist_filter`
    https://github.com/LorenzSchueler/dist-iter/blob/885638d42b080fc7033797be07ee9cc0a1c2fea7/dist-iter/examples/readme-provided-methods.rs#L10-L13
- `dist_reduce`
    https://github.com/LorenzSchueler/dist-iter/blob/885638d42b080fc7033797be07ee9cc0a1c2fea7/dist-iter/examples/readme-provided-methods.rs#L16-L22
- `dist_for_each`
    https://github.com/LorenzSchueler/dist-iter/blob/885638d42b080fc7033797be07ee9cc0a1c2fea7/dist-iter/examples/readme-provided-methods.rs#L53-L56
- `dist_map_chunk`
    https://github.com/LorenzSchueler/dist-iter/blob/885638d42b080fc7033797be07ee9cc0a1c2fea7/dist-iter/examples/readme-provided-methods.rs#L25-L39
    https://github.com/LorenzSchueler/dist-iter/blob/885638d42b080fc7033797be07ee9cc0a1c2fea7/dist-iter/examples/readme-provided-methods.rs#L42-L50
- For all adapters which return an iterator (currently `dist_map`, `dist_filter` and `dist_map_chunk`) there is also a variant called `*_collect` (e.g. `dist_map_collect`) which instead of returning an iterator, returns a `std::vec::Vec`. 
    This should be preferred if the next call would be `collect::<Vec<_>>()` anyway because it avoids writing the data in the receive-buffer and then copying it into the vector. 
    Instead, the received data is directly written into the vector.
    Unlike the `collect` method in `std::iter::Iterator` the collect variant is not generic over the collection type.
    In order to collect into a different collection the normal variant followed by a call to `collect` has to be used.
    https://github.com/LorenzSchueler/dist-iter/blob/885638d42b080fc7033797be07ee9cc0a1c2fea7/dist-iter/examples/readme-provided-methods.rs#L59-L60
- Most of the methods in `std::iter::Iterator` make no sense in this context because it will be cheaper to execute them on the master rank instead of sending the data to a worker and then sending back the result. 

    Others *might* be useful, but can be simulated easily by one of the `dist_*` adapters:
    - `find(predicate)` → `dist_filter(predicate).next()`
    - `sum()` → `dist_reduce(std::ops::Add::add).unwrap_or_default()`
    - `max()` → `dist_reduce(std::cmp::max)`
    - `min()` → `dist_reduce(std::cmp::min)`
    - `all(predicate)` → `dist_filter(!predicate).next().is_none()`
    - `any(predicate)` → `dist_filter(predicate).next().is_some()`

## Multiple Parallel Adapters

Since the return value of the `dist_*` methods are Iterators themselves, multiple parallel adapters can be applied after another.

https://github.com/LorenzSchueler/dist-iter/blob/885638d42b080fc7033797be07ee9cc0a1c2fea7/dist-iter/examples/readme-multiple-adapters.rs#L6-L8

However, in this case the results of `dist_map` are sent back to the master rank and then the received results are sent out again to worker ranks as input for `dist_filter`.

If this is not intended, and both closures should be applied at the worker before sending back the result `dist_map_chunk` can be used.

https://github.com/LorenzSchueler/dist-iter/blob/885638d42b080fc7033797be07ee9cc0a1c2fea7/dist-iter/examples/readme-multiple-adapters.rs#L11-L17

## Chunk Sizes

Chunk sizes greatly influence the performance of the program.
The larger the chunks, the fewer message transfers are necessary.
However, big (and few) chunks make work balancing worse if the processing time is data dependent.

Chunk sizes have to be known at compile time, in order to be able to avoid allocations. 
The master rank will always fill the chunks completely, unless the underlying iterator is finished.

On the worker side it depends on the kind of adapter:
- Some adapters (like `dist_map`) do not modify the number of items. 
Therefore, the input and output chunk sizes are the same and the chunks which get returned will be completely filled.
- Others (like `dist_filter`) modify the number of items. 
The input and output chunk sizes are still the same because the number of output items *can* be the same as the number of input items.
However, the response *may* contain fewer items.
- Adapters like `dist_reduce` return only a single value.
- `dist_for_each` returns no elements but just sends an empty message to notify the master that the current chunk is finished, and the next chunk can be sent.
- `dist_map_chunk` can be used to implement every other adapter. 
In fact many, but not all, are implemented using `dist_map_chunk`.
The number of returned items can be lower or higher than the number of received items.
Therefore, the input and output chunk size can be specified separately.

## Install

```sh
sudo apt install libopenmpi-dev
cargo install cargo-mpirun
```
