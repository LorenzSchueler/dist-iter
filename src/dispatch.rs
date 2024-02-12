use mpi::Tag;

use crate::{
    functions::{FIBONACCI, SQUARE},
    traits::Function,
    universe_guard::END,
};

const FUNCTIONS: [&dyn Function; 3] = [&END, &FIBONACCI, &SQUARE]; // must be sorted and tags ascending without gaps

pub fn tag_to_function(tag: Tag) -> &'static dyn Function {
    FUNCTIONS[tag as usize]
}
