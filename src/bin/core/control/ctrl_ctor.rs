use crate::core::syntax::*;

pub trait ControlNodeCtor {
    fn add_child(&mut self, instr: SyntaxNode);
    fn next_branch(&mut self);
    fn node(&self) -> SyntaxNode;
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

pub struct IfCtor {
    pub i: If,
    pub branch_idx: i32,
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
