use super::nodes::*;
use super::tree::SyntaxTree;

impl SyntaxTree for SyntaxNode {
    fn dump(&self) -> String {
        match self {
            SyntaxNode::Unparsed(n) => n.dump(),
            SyntaxNode::Module(n) => n.dump(),
            SyntaxNode::Func(n) => n.dump(),
            SyntaxNode::Move(n) => n.dump(),
            SyntaxNode::CallExt(n) => n.dump(),
            SyntaxNode::GcBif(n) => n.dump(),
            SyntaxNode::PutList(n) => n.dump(),
            SyntaxNode::Tr(n) => n.dump(),
            SyntaxNode::Test(n) => n.dump(),
            SyntaxNode::CallExtLast(n) => n.dump(),
            SyntaxNode::CallExtOnly(n) => n.dump(),
            SyntaxNode::Label(n) => n.dump(),
            SyntaxNode::XReg(n) => n.dump(),
            SyntaxNode::YReg(n) => n.dump(),
            SyntaxNode::ExtFunc(n) => n.dump(),
            SyntaxNode::FLabel(n) => n.dump(),
            SyntaxNode::Literal(n) => n.dump(),
            SyntaxNode::Integer(n) => n.dump(),
            SyntaxNode::Nil(n) => n.dump(),
            SyntaxNode::Atom(n) => n.dump(),
            SyntaxNode::TInteger(n) => n.dump(),
            SyntaxNode::Line(n) => n.dump(),
            SyntaxNode::FuncInfo(n) => n.dump(),
            SyntaxNode::Allocate(n) => n.dump(),
            SyntaxNode::TestHeap(n) => n.dump(),
            SyntaxNode::InitYRegs(n) => n.dump(),
            SyntaxNode::IsGe(n) => n.dump(),
            SyntaxNode::IsEqExact(n) => n.dump(),
            SyntaxNode::If(n) => n.dump(),
            SyntaxNode::InstrSeq(n) => n.dump(),
            SyntaxNode::Goto(n) => n.dump(),
        }
    }
}

impl SyntaxTree for Unparsed {
    fn dump(&self) -> String {
        format!(
            "{:?}\n^^^^^^^^^^^^^^^^^^^^^^^^^\n{}\n\n",
            self.term, self.reason
        )
    }
}

impl SyntaxTree for Module {
    fn dump(&self) -> String {
        let mut tr = vec![format!("-module({}).\n", self.name)];
        for func in &self.funcs {
            let func_tr = func.dump();

            tr.push(func_tr)
        }
        tr.join("\n\n")
    }
}

impl SyntaxTree for Func {
    fn dump(&self) -> String {
        let args = (0..self.arity)
            .map(|x| format!("X{}", x))
            .collect::<Vec<_>>()
            .join(", ");
        let mut tr = vec![format!("{}({}) ->", self.name, args)];
        for instr in &self.instrs {
            let instr_tr = instr.dump();
            let instr_tr_in_func =
                String::from("    ") + instr_tr.as_str().replace("\n", "\n    ").as_str();
            tr.push(instr_tr_in_func);
        }
        tr.join("\n")
    }
}

impl SyntaxTree for Label {
    fn dump(&self) -> String {
        format!("\nlabel{}:", self.num)
    }
}

impl SyntaxTree for XReg {
    fn dump(&self) -> String {
        format!("X{}", self.num)
    }
}

impl SyntaxTree for YReg {
    fn dump(&self) -> String {
        format!("Y{}", self.num)
    }
}

impl SyntaxTree for Move {
    fn dump(&self) -> String {
        let tr1 = self.lvalue.dump();
        let tr2 = self.rvalue.dump();
        format!("{} = {}", tr1, tr2)
    }
}

impl SyntaxTree for CallExt {
    fn dump(&self) -> String {
        format!("X0 = {}", self.func.dump())
    }
}

impl SyntaxTree for ExtFunc {
    fn dump(&self) -> String {
        let args = (0..self.arity)
            .map(|x| format!("X{}", x))
            .collect::<Vec<_>>()
            .join(", ");
        format!("{}:{}({})", self.module, self.name, args)
    }
}

impl SyntaxTree for FLabel {
    fn dump(&self) -> String {
        format!("label{}", self.num)
    }
}

impl SyntaxTree for GcBif {
    fn dump(&self) -> String {
        let args = self
            .args
            .iter()
            .map(|x| x.dump())
            .collect::<Vec<_>>()
            .join(", ");
        let store_tr = self.store.dump();
        format!("{} = {}({})", store_tr, self.name, args)
    }
}

impl SyntaxTree for Literal {
    fn dump(&self) -> String {
        format!("{:?}", self.s)
    }
}

impl SyntaxTree for CallExtLast {
    fn dump(&self) -> String {
        format!("return {}", self.func.dump())
    }
}

impl SyntaxTree for CallExtOnly {
    fn dump(&self) -> String {
        format!("return {}", self.func.dump())
    }
}

impl SyntaxTree for Integer {
    fn dump(&self) -> String {
        format!("{}", self.num)
    }
}

impl SyntaxTree for PutList {
    fn dump(&self) -> String {
        let head_tr = self.head.dump();
        let store_tr = self.store.dump();
        if let SyntaxNode::Nil(_) = *self.tail {
            format!("{} = [{}]", store_tr, head_tr)
        } else {
            let tail_tr = self.tail.dump();
            format!("{} = [{} | {}]", store_tr, head_tr, tail_tr)
        }
    }
}

impl SyntaxTree for Atom {
    fn dump(&self) -> String {
        self.name.clone()
    }
}

impl SyntaxTree for TInteger {
    fn dump(&self) -> String {
        format!("{} - {}", self.num1, self.num2)
    }
}

impl SyntaxTree for Tr {
    fn dump(&self) -> String {
        let reg_tr = self.reg.dump();
        format!("{}", reg_tr)
    }
}

impl SyntaxTree for Test {
    fn dump(&self) -> String {
        let tr_comp = self.comp.dump();
        let tr_fail = self.fail.dump();
        let tr_args = self
            .args
            .iter()
            .map(|x| x.dump())
            .collect::<Vec<_>>()
            .join(", ");
        format!("\nif {}({}) fail then goto {}", tr_comp, tr_args, tr_fail)
    }
}

impl SyntaxTree for Line {
    fn dump(&self) -> String {
        format!("# line {}", self.num)
    }
}

impl SyntaxTree for If {
    fn dump(&self) -> String {
        let t = String::from("    ") + self.t.dump().as_str().replace("\n", "\n    ").as_str();
        let f = String::from("    ") + self.f.dump().as_str().replace("\n", "\n    ").as_str();
        let comp = self.comp.dump();
        let args = self
            .args
            .iter()
            .map(|x| x.dump())
            .collect::<Vec<_>>()
            .join(", ");
        format!("if {}({}) {{\n{}\n}} else {{\n{}\n}}", comp, args, t, f)
    }
}

impl SyntaxTree for InstrSeq {
    fn dump(&self) -> String {
        let mut result: Vec<String> = Vec::with_capacity(self.instrs.len());
        for instr in &self.instrs {
            result.push(instr.dump());
        }
        result.join("\n")
    }
}

impl SyntaxTree for Goto {
    fn dump(&self) -> String {
        format!("goto {}", self.num)
    }
}

impl SyntaxTree for Nil {
    fn dump(&self) -> String {
        Nil::LEXEME.into()
    }
}

impl SyntaxTree for IsGe {
    fn dump(&self) -> String {
        IsGe::LEXEME.into()
    }
}

impl SyntaxTree for IsEqExact {
    fn dump(&self) -> String {
        IsEqExact::LEXEME.into()
    }
}

impl SyntaxTree for TestHeap {
    fn dump(&self) -> String {
        format!("# {}", TestHeap::LEXEME)
    }
}

impl SyntaxTree for InitYRegs {
    fn dump(&self) -> String {
        format!("# {}", InitYRegs::LEXEME)
    }
}

impl SyntaxTree for Allocate {
    fn dump(&self) -> String {
        format!("# {}", Allocate::LEXEME)
    }
}

impl SyntaxTree for FuncInfo {
    fn dump(&self) -> String {
        format!("# {}", FuncInfo::LEXEME)
    }
}
