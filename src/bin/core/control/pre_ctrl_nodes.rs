use crate::core::control::ctrl_ctor::*;
use crate::core::syntax::*;

pub enum Branch {
    Next,
    Stop,
    Label(i32),
    Fallthrough(Box<dyn ControlNodeCtor>, Vec<i32>),
    Branch(Box<dyn ControlNodeCtor>, Vec<i32>),
}

pub trait PreControlNode {
    fn translate(&self) -> Result<Branch, &'static str>;
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
