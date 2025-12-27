use crate::core::control::graph::{BBlock, ControlNodeCtor, Merge};
use crate::core::syntax::{Goto, InstrSeq, Label, SyntaxNode};

fn _cfg_to_st(blocks: &mut Vec<BBlock>, visited: &mut Vec<bool>, idx: usize) -> SyntaxNode {
    if visited[idx] {
        let label = match blocks[idx]
            .merge
            .as_ref()
            .expect("Entering a node which has no merge info second time")
        {
            Merge::Final { label_num } => label_num,
            _ => panic!("Entering a merging node which did not finalized"),
        };
        return SyntaxNode::Goto(Goto { num: *label });
    }

    visited[idx] = true;

    let mut instrs: Vec<SyntaxNode> = Vec::new();
    if let Some(merge) = &blocks[idx].merge {
        match merge {
            Merge::Final { label_num } => instrs.push(SyntaxNode::Label(Label { num: *label_num })),
            _ => panic!("Entering a merging node which did not finalized"),
        }
    };

    instrs.extend(std::mem::take(&mut blocks[idx].instrs));

    let succ_option = blocks[idx].successor;
    let mut succ_node: Option<SyntaxNode> = None;
    if let Some(succ) = succ_option {
        succ_node = Some(_cfg_to_st(blocks, visited, succ));
        visited[succ] = true;
    }

    let ctrl_option = &mut blocks[idx].ctrl.take();
    if let Some(ctrl) = ctrl_option {
        let children_len = blocks[idx].children.len();
        for i in 0..children_len {
            let subnode = _cfg_to_st(blocks, visited, blocks[idx].children[i]);
            ctrl.add_child(subnode);
            ctrl.next_branch();
        }
        instrs.push(ctrl.node());
    } else {
        let children_len = blocks[idx].children.len();
        if children_len > 1 {
            panic!("Entering a node with no control instruction but with multiple children")
        }
        if children_len != 0 {
            let node = _cfg_to_st(blocks, visited, blocks[idx].children[0]);
            instrs.push(node);
        }
    }

    if let Some(node) = succ_node {
        instrs.push(node);
    }

    SyntaxNode::InstrSeq(InstrSeq { instrs })
}

pub fn cfg_to_st(blocks: &mut Vec<BBlock>) -> SyntaxNode {
    if blocks.is_empty() {
        return SyntaxNode::InstrSeq(InstrSeq { instrs: Vec::new() });
    }

    let mut visited = vec![false; blocks.len()];
    _cfg_to_st(blocks, &mut visited, 0)
}
