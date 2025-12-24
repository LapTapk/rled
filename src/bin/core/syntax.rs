use erlang::OtpErlangTerm;

pub trait SyntaxTree {
    fn dump(&self) -> String;
}

macro_rules! info_node {
    {$type:ident, $s:literal} => {
        #[derive(Clone, Debug)]
        pub struct $type;
        impl SyntaxTree for $type {
            fn dump(&self) -> String {
                format!("# {}", $s)
            }
        }
    };
}

macro_rules! node_as_lexeme {
    {$type:ident, $s:literal} => {
        #[derive(Clone, Debug)]
        pub struct $type;
        impl SyntaxTree for $type {
            fn dump(&self) -> String {
                $s.into()
            }
        }
    };
}

macro_rules! construct_syntax_node {
    ( $( $node:ident ),*  $(,)? ) => {
        #[derive(Clone, Debug)]
        pub enum SyntaxNode {
        $(
            $node($node),
        )*
        }

        impl SyntaxTree for SyntaxNode {
            fn dump(&self) -> String {
                match self {
                $(
                    SyntaxNode::$node(n) => n.dump(),
                )*
                }
            }
        }
    }
}

construct_syntax_node!(
    Unparsed,
    Module,
    Func,
    Move,
    CallExt,
    GcBif,
    PutList,
    Tr,
    Test,
    CallExtLast,
    CallExtOnly,
    Label,
    XReg,
    YReg,
    ExtFunc,
    FLabel,
    Literal,
    Integer,
    Nil,
    Atom,
    TInteger,
    Line,
    FuncInfo,
    Allocate,
    TestHeap,
    InitYRegs,
    IsGe,
    IsEqExact,
    If,
    InstrSeq
);

#[derive(Clone, Debug)]
pub struct Unparsed {
    pub term: OtpErlangTerm,
    pub reason: String,
}

impl Unparsed {
    pub fn new(term: &OtpErlangTerm, reason: &str) -> SyntaxNode {
        SyntaxNode::Unparsed(Unparsed {
            term: term.clone(),
            reason: reason.into(),
        })
    }
}

impl SyntaxTree for Unparsed {
    fn dump(&self) -> String {
        let tr = format!(
            "{:?}\n^^^^^^^^^^^^^^^^^^^^^^^^^\n{}\n\n",
            self.term, self.reason
        );
        tr
    }
}

#[derive(Clone, Debug)]
pub struct Module {
    pub name: String,
    pub funcs: Vec<SyntaxNode>,
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

#[derive(Clone, Debug)]
pub struct Func {
    pub name: String,
    pub arity: i32,
    pub label: i32,
    pub instrs: Vec<SyntaxNode>,
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

#[derive(Clone, Debug)]
pub struct Label {
    pub num: i32,
}

impl SyntaxTree for Label {
    fn dump(&self) -> String {
        format!("\nlabel{}:", self.num)
    }
}

#[derive(Clone, Debug)]
pub struct XReg {
    pub num: i32,
}

impl SyntaxTree for XReg {
    fn dump(&self) -> String {
        format!("X{}", self.num)
    }
}

#[derive(Clone, Debug)]
pub struct YReg {
    pub num: i32,
}

impl SyntaxTree for YReg {
    fn dump(&self) -> String {
        format!("Y{}", self.num)
    }
}

#[derive(Clone, Debug)]
pub struct Move {
    pub lvalue: Box<SyntaxNode>,
    pub rvalue: Box<SyntaxNode>,
}

impl SyntaxTree for Move {
    fn dump(&self) -> String {
        let tr1 = self.lvalue.dump();
        let tr2 = self.rvalue.dump();
        format!("{} = {}", tr1, tr2)
    }
}

#[derive(Clone, Debug)]
pub struct CallExt {
    pub arity: i32,
    pub func: Box<SyntaxNode>,
}

impl SyntaxTree for CallExt {
    fn dump(&self) -> String {
        format!("X0 = {}", self.func.dump())
    }
}

#[derive(Clone, Debug)]
pub struct ExtFunc {
    pub module: String,
    pub name: String,
    pub arity: i32,
}

impl SyntaxTree for ExtFunc {
    fn dump(&self) -> String {
        let args = (0..self.arity)
            .map(|x| format!("X{}", x))
            .collect::<Vec<_>>()
            .join(", ");
        let tr = format!("{}:{}({})", self.module, self.name, args);
        tr
    }
}

#[derive(Clone, Debug)]
pub struct FLabel {
    pub num: i32,
}

impl SyntaxTree for FLabel {
    fn dump(&self) -> String {
        format!("label{}", self.num)
    }
}

#[derive(Clone, Debug)]
pub struct GcBif {
    pub name: String,
    pub fallback: Box<SyntaxNode>,
    pub arity: i32,
    pub args: Vec<SyntaxNode>,
    pub store: Box<SyntaxNode>,
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
        let tr = format!("{} = {}({})", store_tr, self.name, args);
        tr
    }
}

#[derive(Clone, Debug)]
pub struct Literal {
    pub s: String,
}

impl SyntaxTree for Literal {
    fn dump(&self) -> String {
        format!("{:?}", self.s)
    }
}

#[derive(Clone, Debug)]
pub struct CallExtLast {
    pub arity: i32,
    pub func: Box<SyntaxNode>,
}

impl SyntaxTree for CallExtLast {
    fn dump(&self) -> String {
        format!("return {}", self.func.dump())
    }
}

#[derive(Clone, Debug)]
pub struct CallExtOnly {
    pub arity: i32,
    pub func: Box<SyntaxNode>,
}

impl SyntaxTree for CallExtOnly {
    fn dump(&self) -> String {
        format!("return {}", self.func.dump())
    }
}

#[derive(Clone, Debug)]
pub struct Integer {
    pub num: i32,
}

impl SyntaxTree for Integer {
    fn dump(&self) -> String {
        format!("{}", self.num)
    }
}

#[derive(Clone, Debug)]
pub struct PutList {
    pub head: Box<SyntaxNode>,
    pub tail: Box<SyntaxNode>,
    pub store: Box<SyntaxNode>,
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

#[derive(Clone, Debug)]
pub struct Atom {
    pub name: String,
}

impl SyntaxTree for Atom {
    fn dump(&self) -> String {
        self.name.clone()
    }
}

#[derive(Clone, Debug)]
pub struct TInteger {
    pub num1: i32,
    pub num2: i32,
}

impl SyntaxTree for TInteger {
    fn dump(&self) -> String {
        format!("{} - {}", self.num1, self.num2)
    }
}

#[derive(Clone, Debug)]
pub struct Tr {
    pub reg: Box<SyntaxNode>,
    pub ty: Box<SyntaxNode>,
}

impl SyntaxTree for Tr {
    fn dump(&self) -> String {
        let reg_tr = self.reg.dump();
        format!("{}", reg_tr)
    }
}

#[derive(Clone, Debug)]
pub struct Test {
    pub comp: Box<SyntaxNode>,
    pub fail: Box<SyntaxNode>,
    pub args: Vec<SyntaxNode>,
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

#[derive(Clone, Debug)]
pub struct Line {
    pub num: i32,
}

impl SyntaxTree for Line {
    fn dump(&self) -> String {
        format!("# line {}", self.num)
    }
}

#[derive(Clone, Debug)]
pub struct If {
    pub t: InstrSeq,
    pub f: InstrSeq,
    pub comp: Box<SyntaxNode>,
    pub args: Vec<SyntaxNode>,
}

impl SyntaxTree for If {
    fn dump(&self) -> String {
        let t = self.t.dump();
        let f = self.f.dump();
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

#[derive(Clone, Debug)]
pub struct InstrSeq {
    pub instrs: Vec<SyntaxNode>,
}

impl SyntaxTree for InstrSeq {
    fn dump(&self) -> String {
        let mut result: Vec<String> = Vec::with_capacity(self.instrs.len());
        for instr in &self.instrs {
            result.push(
                String::from("    ") + instr.dump().as_str().replace("\n", "\n    ").as_str(),
            );
        }
        result.join("\n")
    }
}

node_as_lexeme! {Nil, "nil"}
node_as_lexeme! {IsGe, "is_ge"}
node_as_lexeme! {IsEqExact, "is_eq_exact"}

info_node! {TestHeap, "test_heap"}
info_node! {InitYRegs, "init_yregs"}
info_node! {Allocate, "allocate"}
info_node! {FuncInfo, "func_info"}
