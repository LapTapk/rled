use crate::core::control::bblock::*;

pub fn cfg_merges(blocks: &mut Vec<BBlock>) {
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
