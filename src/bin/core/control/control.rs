use crate::core::syntax::*;
use std::collections::{HashMap, VecDeque};

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
    fn add_children(&mut self, instrs: Vec<SyntaxNode>);
    fn next_branch(&mut self);
    fn node(&self) -> SyntaxNode;
}

struct BBlock {
    parent_distance: i32,
    instrs: Vec<SyntaxNode>,
    children: Vec<usize>,
    control_instr: Option<Box<dyn ControlNodeCtor>>,
}

impl BBlock {
    fn new() -> BBlock {
        BBlock {
            parent_distance: 0,
            instrs: Vec::new(),
            children: Vec::new(),
            control_instr: None,
        }
    }
}

fn cfg_no_parent(instrs: Vec<SyntaxNode>) -> Vec<BBlock> {
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
                cur_block.control_instr = Some(ctor);
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
                cur_block.control_instr = Some(ctor);

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

fn cfg_parents(blocks: &mut Vec<BBlock>) {
    if blocks.len() <= 1 {
        return;
    }

    let mut bfs_deque: VecDeque<usize> = VecDeque::new();
    bfs_deque.extend(&blocks[0].children);

    bfs_deque.push_back(0);

    while let Some(u) = bfs_deque.pop_front() {
        let d = blocks[u].parent_distance;
        let children_len = blocks[u].children.len();

        for i in 0..children_len {
            let v = blocks[u].children[i];

            if blocks[v].parent_distance == 0 {
                if blocks[v].control_instr.is_none() {
                    blocks[v].parent_distance = d;
                    bfs_deque.push_front(v);
                } else {
                    blocks[v].parent_distance = d + 1;
                    bfs_deque.push_back(v);
                }

            }
        }
    }
}

fn _cfg_to_st(
    blocks: &mut Vec<BBlock>,
    visited: &mut Vec<bool>,
    block_idx: usize,
    parent_stack: &mut Vec<usize>,
    parent_arena: &mut Vec<Box<dyn ControlNodeCtor>>,
) {
    let (parent_distance, instrs, control_option, children) = {
        let block = &mut blocks[block_idx];

        let parent_distance = block.parent_distance;
        let instrs = std::mem::take(&mut block.instrs);
        let children = std::mem::take(&mut block.children);
        let control_option = block.control_instr.take();

        (parent_distance, instrs, control_option, children)
    };
    visited[block_idx] = true;

    parent_stack.truncate(1 + parent_distance as usize);

    let Some(&parent_control_idx) = parent_stack.last() else {
        panic!("parent_stack became empty");
    };

    parent_arena[parent_control_idx].add_children(instrs);

    let Some(control) = control_option else {
        if children.is_empty() {
            return;
        }
        let next = children[0];
        if visited[next] {
            return;
        }

        _cfg_to_st(blocks, visited, next, parent_stack, parent_arena);
        return;
    };

    parent_arena.push(control);
    let control_idx = parent_arena.len() - 1;
    parent_stack.push(control_idx);

    for i in children {
        if visited[i] {
            continue;
        }

        _cfg_to_st(blocks, visited, i, parent_stack, parent_arena);
        parent_arena[control_idx].next_branch();
    }

    let parent_stack_len = parent_stack.len();
    if parent_stack_len >= parent_distance as usize {
        parent_stack.truncate(1 + parent_distance as usize);
    }

    let node = parent_arena[control_idx].node();
    parent_arena[parent_control_idx].add_child(node);
}

fn cfg_to_st(mut blocks: Vec<BBlock>) -> SyntaxNode {
    if blocks.len() == 0 {
        return SyntaxNode::InstrSeq(InstrSeq { instrs: Vec::new() });
    }

    let mut visited = vec![false; blocks.len()];
    let mut parent_stack = vec![0 as usize];
    let mut parent_arena: Vec<Box<dyn ControlNodeCtor>> =
        vec![Box::new(InstrSeq { instrs: Vec::new() })];

    _cfg_to_st(
        &mut blocks,
        &mut visited,
        0,
        &mut parent_stack,
        &mut parent_arena,
    );

    parent_arena[0].node()
}

pub fn cfg(instrs: Vec<SyntaxNode>) -> SyntaxNode {
    let mut blocks = cfg_no_parent(instrs);
    cfg_parents(&mut blocks);
    cfg_to_st(blocks)
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

    fn add_children(&mut self, instrs: Vec<SyntaxNode>) {
        self.instrs.extend(instrs);
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

    fn add_children(&mut self, instrs: Vec<SyntaxNode>) {
        if self.branch_idx == 0 {
            self.i.t.instrs.extend(instrs);
        } else {
            self.i.f.instrs.extend(instrs);
        }
    }

    fn next_branch(&mut self) {
        self.branch_idx += 1;
    }

    fn node(&self) -> SyntaxNode {
        SyntaxNode::If(self.i.clone())
    }
}
