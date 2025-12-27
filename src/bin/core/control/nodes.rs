use crate::core::control::graph::{Branch, ControlNodeCtor, PreControlNode};
use crate::core::syntax::*;

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
