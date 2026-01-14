use crate::core::control::bblock::*;
use crate::core::control::transform::merges::cfg_merges;

fn _cfg_analysis(
    blocks: &mut Vec<BBlock>,
    visited: &mut Vec<bool>,
    parents: &mut Vec<usize>,
    parents_stack: &mut Vec<usize>,
    idx: usize,
) {
    let mut merge_option = blocks[idx].merge.take();
    if let Some(merge) = &mut merge_option {
        match merge {
            Merge::NotVisited { parent_cnt } => {
                *merge = Merge::Merging {
                    remains: *parent_cnt - 1,
                    parents_stack: parents_stack.clone(),
                }
            }
            Merge::Merging {
                remains,
                parents_stack: saved_parents_stack,
            } => {
                let last_common_fork_idx = parents_stack
                    .iter()
                    .zip(saved_parents_stack.iter())
                    .take_while(|(x, y)| x == y)
                    .count();

                saved_parents_stack.truncate(last_common_fork_idx);
                *remains -= 1;

                if *remains == 0 {
                    let last_common_fork = saved_parents_stack[last_common_fork_idx - 1];
                    blocks[last_common_fork].successor = Some(idx);
                    *merge = Merge::Final;
                }

                blocks[idx].merge = merge_option;
                return;
            }
            Merge::Final => {
                panic!("Entering a merged node");
            }
        }
    }

    blocks[idx].merge = merge_option;

    if visited[idx] {
        return;
    }

    visited[idx] = true;

    parents.push(idx);
    parents_stack.push(parents.len() - 1);

    let children_len = blocks[idx].children.len();
    for c_idx in 0..children_len {
        let c = blocks[idx].children[c_idx];
        _cfg_analysis(blocks, visited, parents, parents_stack, c);
    }

    parents_stack.pop();
}

pub fn cfg_analysis(blocks: &mut Vec<BBlock>) {
    cfg_merges(blocks);

    let mut visited = vec![false; blocks.len()];
    let mut parents: Vec<usize> = Vec::new();
    let mut parents_stack: Vec<usize> = Vec::new();

    _cfg_analysis(blocks, &mut visited, &mut parents, &mut parents_stack, 0);
}
