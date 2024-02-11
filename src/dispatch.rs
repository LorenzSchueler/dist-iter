use mpi::Tag;

use crate::{
    functions::{Fibonacci, Square},
    traits::Function,
    universe_guard::End,
};

const END: End = End {};
const FIBONACCI: Fibonacci = Fibonacci {};
const SQUARE: Square = Square {};
const FUNCTIONS: [&dyn Function; 3] = [&END, &FIBONACCI, &SQUARE]; // must be sorted and tags ascending without gaps

pub fn tag_to_function(tag: Tag) -> &'static dyn Function {
    FUNCTIONS[tag as usize]
}
