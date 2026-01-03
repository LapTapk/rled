use crate::core::control::bblock::*;
use crate::core::syntax::*;

fn _cfg_to_st(
    blocks: &mut Vec<BBlock>,
    visited: &mut Vec<bool>,
    parent_succ: usize,
    last_label: &mut i32,
    idx: usize,
) -> Option<SyntaxNode> {
    if visited[idx] {
        if parent_succ == idx {
            return None;
        }

        if blocks[idx].label.is_none() {
            blocks[idx].label = Some(*last_label);
            *last_label += 1;
        }

        let label = blocks[idx].label.unwrap();
        return Some(SyntaxNode::Goto(Goto { num: label }));
    }

    visited[idx] = true;

    let mut instrs: Vec<SyntaxNode> = Vec::new();
    instrs.extend(std::mem::take(&mut blocks[idx].instrs));

    let mut cur_succ = parent_succ;
    let succ_option = blocks[idx].successor;
    let mut succ_node_option = None;
    if let Some(succ) = succ_option {
        succ_node_option = _cfg_to_st(blocks, visited, parent_succ, last_label, succ);
        cur_succ = succ;
        visited[succ] = true;
    }

    let ctrl_option = &mut blocks[idx].ctrl.take();
    if let Some(ctrl) = ctrl_option {
        let children_len = blocks[idx].children.len();
        for i in 0..children_len {
            let subnode_option = _cfg_to_st(
                blocks,
                visited,
                cur_succ,
                last_label,
                blocks[idx].children[i],
            );
            if let Some(subnode) = subnode_option {
                ctrl.add_child(subnode);
            }
            ctrl.next_branch();
        }
        instrs.push(ctrl.node());
    } else {
        let children_len = blocks[idx].children.len();
        if children_len > 1 {
            panic!(
                "Entering a node with no control instruction but with multiple children.\n{}\n{:?}",
                idx, blocks
            )
        }
        if children_len != 0 {
            let node_option = _cfg_to_st(
                blocks,
                visited,
                cur_succ,
                last_label,
                blocks[idx].children[0],
            );
            if let Some(node) = node_option {
                instrs.push(node);
            }
        }
    }
    if let Some(succ_node) = succ_node_option {
        if let Some(label) = blocks[succ_option.unwrap()].label {
            instrs.push(SyntaxNode::Label(Label { num: label }));
        }
        instrs.push(succ_node);
    }

    Some(SyntaxNode::InstrSeq(InstrSeq { instrs }))
}

pub fn cfg_to_st(blocks: &mut Vec<BBlock>) -> Option<SyntaxNode> {
    if blocks.len() == 0 {
        return None;
    }

    let mut visited = vec![false; blocks.len()];
    let mut last_label = 0;
    _cfg_to_st(blocks, &mut visited, 0, &mut last_label, 0)
}
