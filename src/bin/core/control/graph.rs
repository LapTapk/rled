use crate::core::syntax::*;
use std::collections::HashMap;

pub enum Branch {
    Next,
    Stop,
    Label(i32),
    Fallthrough(Box<dyn ControlNodeCtor>, Vec<i32>),
    Branch(Box<dyn ControlNodeCtor>, Vec<i32>),
}

pub trait PreControlNode {
    fn translate(&self) -> Result<Branch, &'static str>;
}

pub trait ControlNodeCtor {
    fn add_child(&mut self, instr: SyntaxNode);
    fn next_branch(&mut self);
    fn node(&self) -> SyntaxNode;
}

#[derive(Debug)]
pub enum Merge {
    NotVisited {
        parent_cnt: i32,
    },
    Merging {
        remains: i32,
        parents_stack: Vec<usize>,
    },
    Final {
        label_num: i32,
    },
}

pub struct BBlock {
    pub instrs: Vec<SyntaxNode>,
    pub children: Vec<usize>,
    pub successor: Option<usize>,
    pub ctrl: Option<Box<dyn ControlNodeCtor>>,
    pub merge: Option<Merge>,
}

impl BBlock {
    pub fn new() -> BBlock {
        BBlock {
            merge: None,
            successor: None,
            instrs: Vec::new(),
            children: Vec::new(),
            ctrl: None,
        }
    }
}

impl Default for BBlock {
    fn default() -> Self {
        Self::new()
    }
}

pub fn push_block(blocks: &mut Vec<BBlock>, cur_block: &mut BBlock) {
    let old = std::mem::take(cur_block);
    blocks.push(old);
}

pub fn attach_or_enqueue_label(
    cur_block_idx: usize,
    label_num: i32,
    labels_blocks: &HashMap<i32, usize>,
    labels_to_branches: &mut HashMap<i32, Vec<usize>>,
    children: &mut Vec<usize>,
) {
    if let Some(&idx) = labels_blocks.get(&label_num) {
        children.push(idx);
    } else {
        labels_to_branches
            .entry(label_num)
            .or_default()
            .push(cur_block_idx);
    }
}

pub fn link_pending_branches(
    label_num: i32,
    labels_to_branches: &HashMap<i32, Vec<usize>>,
    blocks: &mut [BBlock],
    labeled_block_idx: usize,
) {
    if let Some(block_idxes) = labels_to_branches.get(&label_num) {
        for &idx in block_idxes {
            blocks[idx].children.push(labeled_block_idx);
        }
    }
}
