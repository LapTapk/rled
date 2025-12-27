use erlang::OtpErlangTerm;

macro_rules! info_node {
    {$type:ident, $s:literal} => {
        #[derive(Clone, Debug)]
        pub struct $type;

        impl $type {
            pub const LEXEME: &'static str = $s;
        }
    };
}

macro_rules! node_as_lexeme {
    {$type:ident, $s:literal} => {
        #[derive(Clone, Debug)]
        pub struct $type;

        impl $type {
            pub const LEXEME: &'static str = $s;
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
    InstrSeq,
    Goto
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

#[derive(Clone, Debug)]
pub struct Module {
    pub name: String,
    pub funcs: Vec<SyntaxNode>,
}

#[derive(Clone, Debug)]
pub struct Func {
    pub name: String,
    pub arity: i32,
    pub label: i32,
    pub instrs: Vec<SyntaxNode>,
}

#[derive(Clone, Debug)]
pub struct Label {
    pub num: i32,
}

#[derive(Clone, Debug)]
pub struct XReg {
    pub num: i32,
}

#[derive(Clone, Debug)]
pub struct YReg {
    pub num: i32,
}

#[derive(Clone, Debug)]
pub struct Move {
    pub lvalue: Box<SyntaxNode>,
    pub rvalue: Box<SyntaxNode>,
}

#[derive(Clone, Debug)]
pub struct CallExt {
    pub arity: i32,
    pub func: Box<SyntaxNode>,
}

#[derive(Clone, Debug)]
pub struct ExtFunc {
    pub module: String,
    pub name: String,
    pub arity: i32,
}

#[derive(Clone, Debug)]
pub struct FLabel {
    pub num: i32,
}

#[derive(Clone, Debug)]
pub struct GcBif {
    pub name: String,
    pub fallback: Box<SyntaxNode>,
    pub arity: i32,
    pub args: Vec<SyntaxNode>,
    pub store: Box<SyntaxNode>,
}

#[derive(Clone, Debug)]
pub struct Literal {
    pub s: String,
}

#[derive(Clone, Debug)]
pub struct CallExtLast {
    pub arity: i32,
    pub func: Box<SyntaxNode>,
}

#[derive(Clone, Debug)]
pub struct CallExtOnly {
    pub arity: i32,
    pub func: Box<SyntaxNode>,
}

#[derive(Clone, Debug)]
pub struct Integer {
    pub num: i32,
}

#[derive(Clone, Debug)]
pub struct PutList {
    pub head: Box<SyntaxNode>,
    pub tail: Box<SyntaxNode>,
    pub store: Box<SyntaxNode>,
}

#[derive(Clone, Debug)]
pub struct Atom {
    pub name: String,
}

#[derive(Clone, Debug)]
pub struct TInteger {
    pub num1: i32,
    pub num2: i32,
}

#[derive(Clone, Debug)]
pub struct Tr {
    pub reg: Box<SyntaxNode>,
    pub ty: Box<SyntaxNode>,
}

#[derive(Clone, Debug)]
pub struct Test {
    pub comp: Box<SyntaxNode>,
    pub fail: Box<SyntaxNode>,
    pub args: Vec<SyntaxNode>,
}

#[derive(Clone, Debug)]
pub struct Line {
    pub num: i32,
}

#[derive(Clone, Debug)]
pub struct If {
    pub t: InstrSeq,
    pub f: InstrSeq,
    pub comp: Box<SyntaxNode>,
    pub args: Vec<SyntaxNode>,
}

#[derive(Clone, Debug)]
pub struct InstrSeq {
    pub instrs: Vec<SyntaxNode>,
}

#[derive(Clone, Debug)]
pub struct Goto {
    pub num: i32,
}

node_as_lexeme! {Nil, "nil"}
node_as_lexeme! {IsGe, "is_ge"}
node_as_lexeme! {IsEqExact, "is_eq_exact"}

info_node! {TestHeap, "test_heap"}
info_node! {InitYRegs, "init_yregs"}
info_node! {Allocate, "allocate"}
info_node! {FuncInfo, "func_info"}
