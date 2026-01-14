use crate::core::control::transform::analysis::cfg_analysis;
use crate::core::control::transform::build::cfg_build;
use crate::core::control::transform::to_st::cfg_to_st;
use crate::core::syntax::*;

pub fn cfg(instrs: Vec<SyntaxNode>) -> SyntaxNode {
    let mut blocks = cfg_build(instrs);
    cfg_analysis(&mut blocks);
    match cfg_to_st(&mut blocks) {
        Some(st) => st,
        None => SyntaxNode::InstrSeq(InstrSeq { instrs: Vec::new() }),
    }
}
