use crate::core::syntax::*;
use std::collections::HashMap;

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
    Final {
        label_num: i32,
    },
}

struct BBlock {
    instrs: Vec<SyntaxNode>,
    children: Vec<usize>,

    successor: Option<usize>,
    ctrl: Option<Box<dyn ControlNodeCtor>>,
    merge: Option<Merge>,
}

impl BBlock {
    fn new() -> BBlock {
        BBlock {
            merge: None,
            successor: None,
            instrs: Vec::new(),
            children: Vec::new(),
            ctrl: None,
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

                if let Some(block_idxes) = labels_to_branches.get(&label_num) {
                    for &idx in block_idxes {
                        blocks[idx].children.push(labeled_block_idx);
                    }
                }
                labels_blocks.insert(label_num, labeled_block_idx);
                cur_block.children.push(labeled_block_idx);
                blocks.push(cur_block);
                cur_block = BBlock::new();
            }
            Branch::Branch(ctor, labels_nums) => {
                cur_block.ctrl = Some(ctor);
                let cur_block_idx = blocks.len();

                for &label_num in &labels_nums {
                    if let Some(&idx) = labels_blocks.get(&label_num) {
                        cur_block.children.push(idx);
                    } else {
                        if let None = labels_to_branches.get(&label_num) {
                            labels_to_branches.insert(label_num, Vec::new());
                        }
                        labels_to_branches
                            .get_mut(&label_num)
                            .unwrap()
                            .push(cur_block_idx);
                    }
                }

                blocks.push(cur_block);
                cur_block = BBlock::new();
            }
            Branch::Fallthrough(ctor, labels_nums) => {
                cur_block.ctrl = Some(ctor);

                let cur_block_idx = blocks.len();
                cur_block.children.push(cur_block_idx + 1);

                for &label_num in &labels_nums {
                    if let Some(&idx) = labels_blocks.get(&label_num) {
                        cur_block.children.push(idx);
                    } else {
                        if let None = labels_to_branches.get(&label_num) {
                            labels_to_branches.insert(label_num, Vec::new());
                        }
                        labels_to_branches
                            .get_mut(&label_num)
                            .unwrap()
                            .push(cur_block_idx);
                    }
                }

                blocks.push(cur_block);
                cur_block = BBlock::new();
            }
            Branch::Next => {
                cur_block.instrs.push(instr);
            }
            Branch::Stop => {
                cur_block.instrs.push(instr);
                blocks.push(cur_block);
                cur_block = BBlock::new();
            }
        }
    }
    if !cur_block.instrs.is_empty() {
        blocks.push(cur_block);
    }

    blocks
}

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
                println!("{}: Comparing {:?} and {:?}", idx, parents_stack, saved_parents_stack);
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
                    *merge = Merge::Final {
                        label_num: *last_label_num,
                    };
                    *last_label_num += 1
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

fn cfg_analysis(blocks: &mut Vec<BBlock>) {
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

fn cfg_to_st(blocks: &mut Vec<BBlock>) -> SyntaxNode {
    if blocks.len() == 0 {
        return SyntaxNode::InstrSeq(InstrSeq { instrs: Vec::new() });
    }

    let mut visited = vec![false; blocks.len()];
    _cfg_to_st(blocks, &mut visited, 0)
}

pub fn cfg(instrs: Vec<SyntaxNode>) -> SyntaxNode {
    let mut blocks = cfg_build(instrs);
    cfg_analysis(&mut blocks);
    cfg_to_st(&mut blocks)
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
