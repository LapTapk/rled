use erlang::OtpErlangTerm;

pub enum Reg {
    X { i: i32 },
    Y { i: i32 },
}

pub enum SymbolValue<'a> {
    FuncRes {
        token: &'a dyn Token
    },
    Constant {
        term: &'a OtpErlangTerm  
    },
    Symbol {
        id: i32
    }
}

pub struct Symbol<'a> {
    reg: Reg,
    value: SymbolValue<'a>,
    parent_node: SyntaxTree
}

pub type SymbolStack<'a> = Vec<Symbol<'a>>;

pub enum SyntaxTree {
    Leaf(Box<dyn Token>),
    Node {
        value: Box<dyn Token>,
        children: Vec<SyntaxTree>,
    },
}

pub trait Token {
    fn decode(term: &OtpErlangTerm) -> Result<Self, &'static str>
    where
        Self: Sized;
    //fn update_syntax_tree(&self, st: SyntaxTree) -> SyntaxTree;
}

pub trait Instr: Token {
    fn is_ctrl(&self) -> bool;
}
