use crate::core::syntax::*;
use std::collections::HashMap;
use std::fmt;

enum Branch {
    Next,
    Stop,
    Label(i32),
    Fallthrough(Box<dyn ControlNodeCtor>, Vec<i32>),
    Branch(Box<dyn ControlNodeCtor>, Vec<i32>),
}

trait PreControlNode {
    fn translate(&self) -> Result<Branch, &'static str>;
}

macro_rules! construct_helpers {
    ( $( $node:ident ),* $(,)? ) => {
        impl PreControlNode for SyntaxNode {
            fn translate(&self) -> Result<Branch, &'static str> {
                match self {
                $(
                    SyntaxNode::$node(n) => n.translate(),
                )*
                    _ => Ok(Branch::Next),
                }
            }
        }
    }
}

construct_helpers!(Test, Label, CallExtLast);

trait ControlNodeCtor {
    fn add_child(&mut self, instr: SyntaxNode);
    fn next_branch(&mut self);
    fn node(&self) -> SyntaxNode;
}

#[derive(Debug)]
enum Merge {
    NotVisited {
        parent_cnt: i32,
    },
    Merging {
        remains: i32,
        parents_stack: Vec<usize>,
    },
    Final,
}

struct BBlock {
    instrs: Vec<SyntaxNode>,
    children: Vec<usize>,

    successor: Option<usize>,
    ctrl: Option<Box<dyn ControlNodeCtor>>,
    merge: Option<Merge>,
    label: Option<i32>,
}

impl fmt::Debug for BBlock {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BBlock")
            .field("instrs", &self.instrs)
            .field("children", &self.children)
            .field("successor", &self.successor)
            .field("merge", &self.merge)
            .field("label", &self.label)
            .field("ctrl", &self.ctrl.is_some())
            .finish()
    }
}

impl BBlock {
    fn new() -> BBlock {
        BBlock {
            merge: None,
            successor: None,
            instrs: Vec::new(),
            children: Vec::new(),
            ctrl: None,
            label: None,
        }
    }
}

impl Default for BBlock {
    fn default() -> Self {
        Self::new()
    }
}

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

fn cfg_build(instrs: Vec<SyntaxNode>) -> Vec<BBlock> {
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

fn cfg_analysis(blocks: &mut Vec<BBlock>) {
    cfg_merges(blocks);

    let mut visited = vec![false; blocks.len()];
    let mut parents: Vec<usize> = Vec::new();
    let mut parents_stack: Vec<usize> = Vec::new();

    _cfg_analysis(blocks, &mut visited, &mut parents, &mut parents_stack, 0);
}

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

fn cfg_to_st(blocks: &mut Vec<BBlock>) -> Option<SyntaxNode> {
    if blocks.len() == 0 {
        return None;
    }

    let mut visited = vec![false; blocks.len()];
    let mut last_label = 0;
    _cfg_to_st(blocks, &mut visited, 0, &mut last_label, 0)
}

pub fn cfg(instrs: Vec<SyntaxNode>) -> SyntaxNode {
    let mut blocks = cfg_build(instrs);
    cfg_analysis(&mut blocks);
    println!("{:?}", blocks);
    match cfg_to_st(&mut blocks) {
        Some(st) => st,
        None => SyntaxNode::InstrSeq(InstrSeq { instrs: Vec::new() }),
    }
}

impl PreControlNode for Label {
    fn translate(&self) -> Result<Branch, &'static str> {
        Ok(Branch::Label(self.num))
    }
}

impl PreControlNode for CallExtLast {
    fn translate(&self) -> Result<Branch, &'static str> {
        Ok(Branch::Stop)
    }
}

impl PreControlNode for Test {
    fn translate(&self) -> Result<Branch, &'static str> {
        let SyntaxNode::FLabel(fail) = &*self.fail else {
            return Err("Test's fail field must be Label syntax node");
        };

        let ctor = Box::new(IfCtor {
            i: If {
                t: InstrSeq { instrs: Vec::new() },
                f: InstrSeq { instrs: Vec::new() },
                args: self.args.clone(),
                comp: self.comp.clone(),
            },
            branch_idx: 0,
        });

        Ok(Branch::Fallthrough(ctor, vec![fail.num]))
    }
}

impl ControlNodeCtor for InstrSeq {
    fn add_child(&mut self, instr: SyntaxNode) {
        self.instrs.push(instr);
    }

    fn next_branch(&mut self) {}

    fn node(&self) -> SyntaxNode {
        SyntaxNode::InstrSeq(self.clone())
    }
}

struct IfCtor {
    i: If,
    branch_idx: i32,
}

impl ControlNodeCtor for IfCtor {
    fn add_child(&mut self, instr: SyntaxNode) {
        if self.branch_idx == 0 {
            self.i.t.instrs.push(instr);
        } else {
            self.i.f.instrs.push(instr);
        }
    }

    fn next_branch(&mut self) {
        self.branch_idx += 1;
    }

    fn node(&self) -> SyntaxNode {
        SyntaxNode::If(self.i.clone())
    }
}
