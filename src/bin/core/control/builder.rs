use crate::core::control::graph::{
    BBlock, Branch, PreControlNode, attach_or_enqueue_label, link_pending_branches, push_block,
};
use crate::core::syntax::SyntaxNode;
use std::collections::HashMap;

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
