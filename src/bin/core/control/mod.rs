mod analysis;
mod builder;
mod graph;
mod nodes;
mod reconstruct;

use crate::core::syntax::SyntaxNode;

pub use graph::{BBlock, Branch, ControlNodeCtor, Merge, PreControlNode};

pub fn cfg(instrs: Vec<SyntaxNode>) -> SyntaxNode {
    let mut blocks = builder::cfg_build(instrs);
    analysis::cfg_analysis(&mut blocks);
    reconstruct::cfg_to_st(&mut blocks)
}
