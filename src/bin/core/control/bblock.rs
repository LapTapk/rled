use crate::core::control::ctrl_ctor::ControlNodeCtor;
use crate::core::syntax::*;
use std::fmt;

#[derive(Debug)]
pub enum Merge {
    NotVisited {
        parent_cnt: i32,
    },
    Merging {
        remains: i32,
        parents_stack: Vec<usize>,
    },
    Final,
}

pub struct BBlock {
    pub instrs: Vec<SyntaxNode>,
    pub children: Vec<usize>,

    pub successor: Option<usize>,
    pub ctrl: Option<Box<dyn ControlNodeCtor>>,
    pub merge: Option<Merge>,
    pub label: Option<i32>,
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
    pub fn new() -> BBlock {
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
