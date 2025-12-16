#![allow(unused)]

mod instructions;
mod translation;
mod util;

use erlang::binary_to_term;
use crate::translation::{Module, Token, SymbolList};

fn main() {
    let data = std::fs::read("a").unwrap();
    let term = binary_to_term(&data).unwrap();
    let parsed = <Module as Token>::parse(&term, SymbolList::new()).unwrap().token;
    let tr = parsed.translate();
    let tr_str = tr.join("\n");
    println!("{}", tr_str);
}
