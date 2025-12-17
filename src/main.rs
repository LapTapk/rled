#![allow(unused)]

mod instructions;
mod translation;
mod util;

use crate::translation::*;
use erlang::binary_to_term;

fn main() {
    let data = std::fs::read("a").unwrap();
    let term = binary_to_term(&data).unwrap();
    let parsed = <Module as Token>::parse(&term)
        .unwrap();
    let tr = parsed.translate().unwrap();
    println!("{}", tr);
}
