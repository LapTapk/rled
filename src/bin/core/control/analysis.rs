use crate::core::control::graph::{BBlock, Merge};

fn _cfg_analysis(
    blocks: &mut Vec<BBlock>,
    visited: &mut Vec<bool>,
    parents: &mut Vec<usize>,
    parents_stack: &mut Vec<usize>,
    idx: usize,
    last_label_num: &mut i32,
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
                log::debug!(
                    "{}: comparing merge parents {:?} and {:?}",
                    idx,
                    parents_stack,
                    saved_parents_stack
                );
                let last_common_fork_idx = parents_stack
                    .iter()
                    .zip(saved_parents_stack.iter())
                    .take_while(|(x, y)| x == y)
                    .count();

                saved_parents_stack.truncate(last_common_fork_idx);
                *remains -= 1;

                if *remains == 0 {
                    if let Some(&last_common_fork) =
                        saved_parents_stack.get(last_common_fork_idx.saturating_sub(1))
                    {
                        blocks[last_common_fork].successor = Some(idx);
                        *merge = Merge::Final {
                            label_num: *last_label_num,
                        };
                        *last_label_num += 1;
                    } else {
                        log::warn!(
                            "finalizing merge at {} without common parent: {:?} vs {:?}",
                            idx,
                            parents_stack,
                            saved_parents_stack
                        );
                    }
                }

                blocks[idx].merge = merge_option;
                return;
            }
            Merge::Final { label_num: _ } => {
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
        _cfg_analysis(blocks, visited, parents, parents_stack, c, last_label_num);
    }

    parents_stack.pop();
}

fn cfg_merges(blocks: &mut Vec<BBlock>) {
    let mut parent_cnt = vec![0; blocks.len()];

    for i in 0..blocks.len() {
        for c in 0..blocks[i].children.len() {
            parent_cnt[blocks[i].children[c]] += 1;
        }
    }

    for i in 0..blocks.len() {
        if parent_cnt[i] < 2 {
            continue;
        }

        blocks[i].merge = Some(Merge::NotVisited {
            parent_cnt: parent_cnt[i],
        })
    }
}

pub fn cfg_analysis(blocks: &mut Vec<BBlock>) {
    cfg_merges(blocks);

    let mut visited = vec![false; blocks.len()];
    let mut parents: Vec<usize> = Vec::new();
    let mut parents_stack: Vec<usize> = Vec::new();
    let mut last_label_num = 0;

    _cfg_analysis(
        blocks,
        &mut visited,
        &mut parents,
        &mut parents_stack,
        0,
        &mut last_label_num,
    );
}
