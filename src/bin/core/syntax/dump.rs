use crate::core::syntax::nodes::*;

pub trait SyntaxDump {
    fn dump(&self) -> String;
}

macro_rules! info_node {
    {$type:ident, $s:literal} => {
        impl SyntaxDump for $type {
            fn dump(&self) -> String {
                format!("# {}", $s)
            }
        }
    };
}

macro_rules! node_as_lexeme {
    {$type:ident, $s:literal} => {
        impl SyntaxDump for $type {
            fn dump(&self) -> String {
                $s.into()
            }
        }
    };
}

impl SyntaxDump for Unparsed {
    fn dump(&self) -> String {
        let tr = format!(
            "{:?}\n^^^^^^^^^^^^^^^^^^^^^^^^^\n{}\n\n",
            self.term, self.reason
        );
        tr
    }
}

impl SyntaxDump for Module {
    fn dump(&self) -> String {
        let mut tr = vec![format!("-module({}).\n", self.name)];
        for func in &self.funcs {
            let func_tr = func.dump();

            tr.push(func_tr)
        }
        tr.join("\n\n")
    }
}

impl SyntaxDump for Func {
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

impl SyntaxDump for Label {
    fn dump(&self) -> String {
        format!("\nlabel{}:", self.num)
    }
}

impl SyntaxDump for XReg {
    fn dump(&self) -> String {
        format!("X{}", self.num)
    }
}

impl SyntaxDump for YReg {
    fn dump(&self) -> String {
        format!("Y{}", self.num)
    }
}

impl SyntaxDump for Move {
    fn dump(&self) -> String {
        let tr1 = self.lvalue.dump();
        let tr2 = self.rvalue.dump();
        format!("{} = {}", tr1, tr2)
    }
}

impl SyntaxDump for CallExt {
    fn dump(&self) -> String {
        format!("X0 = {}", self.func.dump())
    }
}

impl SyntaxDump for ExtFunc {
    fn dump(&self) -> String {
        let args = (0..self.arity)
            .map(|x| format!("X{}", x))
            .collect::<Vec<_>>()
            .join(", ");
        let tr = format!("{}:{}({})", self.module, self.name, args);
        tr
    }
}

impl SyntaxDump for FLabel {
    fn dump(&self) -> String {
        format!("label{}", self.num)
    }
}

impl SyntaxDump for GcBif {
    fn dump(&self) -> String {
        let args = self
            .args
            .iter()
            .map(|x| x.dump())
            .collect::<Vec<_>>()
            .join(", ");
        let store_tr = self.store.dump();
        let tr = format!("{} = {}({})", store_tr, self.name, args);
        tr
    }
}

impl SyntaxDump for Literal {
    fn dump(&self) -> String {
        format!("{:?}", self.s)
    }
}

impl SyntaxDump for CallExtLast {
    fn dump(&self) -> String {
        format!("return {}", self.func.dump())
    }
}

impl SyntaxDump for CallExtOnly {
    fn dump(&self) -> String {
        format!("return {}", self.func.dump())
    }
}

impl SyntaxDump for Integer {
    fn dump(&self) -> String {
        format!("{}", self.num)
    }
}

impl SyntaxDump for PutList {
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

impl SyntaxDump for Atom {
    fn dump(&self) -> String {
        self.name.clone()
    }
}

impl SyntaxDump for TInteger {
    fn dump(&self) -> String {
        format!("{} - {}", self.num1, self.num2)
    }
}

impl SyntaxDump for Tr {
    fn dump(&self) -> String {
        let reg_tr = self.reg.dump();
        format!("{}", reg_tr)
    }
}

impl SyntaxDump for Test {
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

impl SyntaxDump for Line {
    fn dump(&self) -> String {
        format!("# line {}", self.num)
    }
}

impl SyntaxDump for If {
    fn dump(&self) -> String {
        let t = String::from("    ") + self.t.dump().as_str().replace("\n", "\n    ").as_str();
        let comp = self.comp.dump();
        let args = self
            .args
            .iter()
            .map(|x| x.dump())
            .collect::<Vec<_>>()
            .join(", ");
        if self.f.instrs.is_empty() {
            format!("if {}({}) {{\n{}\n}}", comp, args, t)
        } else {
            let f = String::from("    ") + self.f.dump().as_str().replace("\n", "\n    ").as_str();
            format!("if {}({}) {{\n{}\n}} else {{\n{}\n}}", comp, args, t, f)
        }
    }
}

impl SyntaxDump for InstrSeq {
    fn dump(&self) -> String {
        let mut result: Vec<String> = Vec::with_capacity(self.instrs.len());
        for instr in &self.instrs {
            result.push(instr.dump());
        }
        result.join("\n")
    }
}

impl SyntaxDump for Goto {
    fn dump(&self) -> String {
        format!("goto {}", self.num)
    }
}

node_as_lexeme! {Nil, "nil"}
node_as_lexeme! {IsGe, "is_ge"}
node_as_lexeme! {IsEqExact, "is_eq_exact"}

info_node! {TestHeap, "test_heap"}
info_node! {InitYRegs, "init_yregs"}
info_node! {Allocate, "allocate"}
info_node! {FuncInfo, "func_info"}
