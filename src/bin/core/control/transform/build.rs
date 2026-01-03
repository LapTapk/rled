use crate::core::control::bblock::*;
use crate::core::control::pre_ctrl_nodes::{Branch, PreControlNode};
use crate::core::syntax::*;
use std::collections::HashMap;

fn push_block(blocks: &mut Vec<BBlock>, cur_block: &mut BBlock) {
    let old = std::mem::take(cur_block);
    blocks.push(old);
}

fn attach_or_enqueue_label(
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

fn link_pending_branches(
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

pub fn cfg_build(instrs: Vec<SyntaxNode>) -> Vec<BBlock> {
    let mut blocks: Vec<BBlock> = Vec::new();
    let mut cur_block = BBlock::new();
    let mut labels_to_branches: HashMap<i32, Vec<usize>> = HashMap::new();
    let mut labels_blocks: HashMap<i32, usize> = HashMap::new();

    for instr in instrs {
        let br = match instr.translate() {
            Ok(b) => b,
            Err(s) => {
                log::warn!("{}", s);
                Branch::Next
            }
        };

        match br {
            Branch::Label(label_num) => {
                let labeled_block_idx = blocks.len() + 1;

                link_pending_branches(
                    label_num,
                    &labels_to_branches,
                    &mut blocks,
                    labeled_block_idx,
                );
                labels_blocks.insert(label_num, labeled_block_idx);
                cur_block.children.push(labeled_block_idx);
                push_block(&mut blocks, &mut cur_block);
            }
            Branch::Branch(ctor, labels_nums) => {
                cur_block.ctrl = Some(ctor);
                let cur_block_idx = blocks.len();

                for &label_num in &labels_nums {
                    attach_or_enqueue_label(
                        cur_block_idx,
                        label_num,
                        &labels_blocks,
                        &mut labels_to_branches,
                        &mut cur_block.children,
                    );
                }

                push_block(&mut blocks, &mut cur_block);
            }
            Branch::Fallthrough(ctor, labels_nums) => {
                cur_block.ctrl = Some(ctor);

                let cur_block_idx = blocks.len();
                cur_block.children.push(cur_block_idx + 1);

                for &label_num in &labels_nums {
                    attach_or_enqueue_label(
                        cur_block_idx,
                        label_num,
                        &labels_blocks,
                        &mut labels_to_branches,
                        &mut cur_block.children,
                    );
                }

                push_block(&mut blocks, &mut cur_block);
            }
            Branch::Next => {
                cur_block.instrs.push(instr);
            }
            Branch::Stop => {
                cur_block.instrs.push(instr);
                push_block(&mut blocks, &mut cur_block);
            }
        }
    }
    if !cur_block.instrs.is_empty() {
        push_block(&mut blocks, &mut cur_block);
    }

    blocks
}
