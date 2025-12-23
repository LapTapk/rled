use crate::core::parse::errors::LexemeError;
use crate::core::parse::grammar::*;
use crate::core::parse::util::atomutf8_to_string;
use erlang::OtpErlangTerm;

pub fn dump(term: &OtpErlangTerm) -> String {
    Module::parse(term).dump()
}

trait ParseTreeCtor {
    const LEXEME: &'static str;
    fn parse(term: &OtpErlangTerm) -> ParseTree;
    fn dump(&self) -> String;
}

pub struct Unparsed {
    pub term: OtpErlangTerm,
    pub reason: String,
}

impl Unparsed {
    fn token(term: &OtpErlangTerm, reason: &str) -> ParseTree {
        log::warn!("Unresolved: {} : {:?}", reason, term);
        ParseTree::Unparsed(Unparsed {
            term: term.clone(),
            reason: reason.into(),
        })
    }
}

impl ParseTreeCtor for Unparsed {
    const LEXEME: &'static str = "";
    fn parse(term: &OtpErlangTerm) -> ParseTree {
        Unparsed::token(term, "ParseTree unregistered")
    }

    fn dump(&self) -> String {
        let tr = format!(
            "{:?}\n^^^^^^^^^^^^^^^^^^^^^^^^^\n{}\n\n",
            self.term, self.reason
        );
        tr
    }
}

macro_rules! try_parse {
    ($term:expr , $e:expr) => {
        match $e {
            Ok(ok) => ok,
            Err(e) => {
                return Unparsed::token($term, format!("{:?}", e).as_str());
            }
        }
    };
}

macro_rules! info_token {
    {$type:ident, $lexeme:literal} => {
        pub struct $type;
        impl ParseTreeCtor for $type {
            const LEXEME: &'static str = $lexeme;
            fn parse(_: &OtpErlangTerm) -> ParseTree {
                ParseTree::$type($type{})
            }

            fn dump(&self) -> String {
                format!("# {}", $lexeme)
            }
        }
    };
}

macro_rules! lexeme_as_token {
    {$type:ident, $lexeme:literal} => {
        pub struct $type;
        impl ParseTreeCtor for $type {
            const LEXEME: &'static str = $lexeme;
            fn parse(_: &OtpErlangTerm) -> ParseTree {
                ParseTree::$type($type{})
            }

            fn dump(&self) -> String {
                $lexeme.into()
            }
        }
    };

}

type ParseFn = fn(&OtpErlangTerm) -> ParseTree;

macro_rules! parse_tree {
    (
        $(
            $token:ident
        ),* $(,)?
    ) => {
        pub enum ParseTree {
            $(
                $token($token),
            )*
            Unparsed(Unparsed),
        }

        impl ParseTree {
            fn dump(&self) -> String {
                match self {
                $(
                    ParseTree::$token(t) => t.dump(),
                )*
                    ParseTree::Unparsed(u) => u.dump()
                }
            }
        }

        fn dispatch(lexeme: &str) -> ParseFn {
            match lexeme {
            $(
                $token::LEXEME => $token::parse,
            )*
                _ => Unparsed::parse,
            }
        }
    };
}

parse_tree!(
    Module,
    Func,
    Label,
    Move,
    XReg,
    YReg,
    CallExt,
    ExtFunc,
    FLabel,
    GcBif,
    Literal,
    CallExtLast,
    CallExtOnly,
    Integer,
    PutList,
    Nil,
    Atom,
    TInteger,
    Tr,
    Test,
    Line,
    FuncInfo,
    Allocate,
    TestHeap,
    InitYRegs,
    IsGe,
    IsEqExact
);

pub fn get_lexeme(term: &OtpErlangTerm) -> Result<String, LexemeError> {
    let atom = match term {
        OtpErlangTerm::OtpErlangTuple(tuple) => {
            if tuple.is_empty() {
                return Err(LexemeError::EmptyTuple);
            }

            &tuple[0]
        }
        OtpErlangTerm::OtpErlangAtomUTF8(_) => term,
        _ => {
            return Err(LexemeError::ExpectedAtomOrTuple {
                found: term.clone(),
            });
        }
    };

    atomutf8_to_string(atom).map_err(|_| LexemeError::Unsupported {
        found: term.clone(),
    })
}

pub fn parse(term: &OtpErlangTerm) -> ParseTree {
    let lexeme = try_parse!(term, get_lexeme(term));
    dispatch(lexeme.as_str())(term)
}

pub struct Module {
    pub name: String,
    pub funcs: Vec<ParseTree>,
}

impl ParseTreeCtor for Module {
    const LEXEME: &'static str = "beam_file";
    fn parse(term: &OtpErlangTerm) -> ParseTree {
        let module = try_parse!(term, expect_tuple(term, 6));
        let name = try_parse!(term, atomutf8_to_string(&module[1]));
        let funcs_beam = try_parse!(term, expect_list(&module[5]));

        let mut funcs: Vec<ParseTree> = Vec::with_capacity(funcs_beam.len());
        for func_beam in funcs_beam {
            let parsed_func = parse(func_beam);
            funcs.push(parsed_func);
        }

        let module = Module { name, funcs };
        ParseTree::Module(module)
    }

    fn dump(&self) -> String {
        let mut tr = vec![format!("-module({}).\n", self.name)];
        for func in &self.funcs {
            let func_tr = func.dump();
            tr.push(func_tr)
        }
        tr.join("\n\n")
    }
}

pub struct Func {
    pub name: String,
    pub arity: i32,
    pub label: i32,
    pub instrs: Vec<ParseTree>,
}

impl ParseTreeCtor for Func {
    const LEXEME: &'static str = "function";
    fn parse(term: &OtpErlangTerm) -> ParseTree {
        let func_tuple = try_parse!(term, expect_tuple(term, 5));
        let name = try_parse!(term, atomutf8_to_string(&func_tuple[1]));
        let arity = try_parse!(term, expect_integer(&func_tuple[2]));
        let label = try_parse!(term, expect_integer(&func_tuple[3]));
        let instrs_list = try_parse!(term, expect_list(&func_tuple[4]));

        let mut instrs: Vec<ParseTree> = Vec::with_capacity(instrs_list.len());
        for instr_term in instrs_list {
            let parsed_instr = parse(instr_term);
            instrs.push(parsed_instr);
        }

        let func = Func {
            name,
            arity,
            label,
            instrs,
        };
        ParseTree::Func(func)
    }

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

pub struct Label {
    pub num: i32,
}

impl ParseTreeCtor for Label {
    const LEXEME: &'static str = "label";
    fn parse(term: &OtpErlangTerm) -> ParseTree {
        let label_tuple = try_parse!(term, expect_tuple(term, 2));

        let OtpErlangTerm::OtpErlangInteger(num) = &label_tuple[1] else {
            return Unparsed::token(term, "Label must have integer as it's 2nd element");
        };

        let label = Label { num: *num };
        ParseTree::Label(label)
    }

    fn dump(&self) -> String {
        format!("\nlabel{}:", self.num)
    }
}

pub struct XReg {
    pub num: i32,
}

impl ParseTreeCtor for XReg {
    const LEXEME: &'static str = "x";
    fn parse(term: &OtpErlangTerm) -> ParseTree {
        let xreg_tuple = try_parse!(term, expect_tuple(term, 2));
        let num = try_parse!(term, expect_integer(&xreg_tuple[1]));

        let xreg = XReg { num };
        ParseTree::XReg(xreg)
    }

    fn dump(&self) -> String {
        format!("X{}", self.num)
    }
}

pub struct YReg {
    pub num: i32,
}

impl ParseTreeCtor for YReg {
    const LEXEME: &'static str = "y";
    fn parse(term: &OtpErlangTerm) -> ParseTree {
        let yreg_tuple = try_parse!(term, expect_tuple(term, 2));
        let num = try_parse!(term, expect_integer(&yreg_tuple[1]));

        let yreg = YReg { num };
        ParseTree::YReg(yreg)
    }

    fn dump(&self) -> String {
        format!("Y{}", self.num)
    }
}

pub struct Move {
    pub lvalue: Box<ParseTree>,
    pub rvalue: Box<ParseTree>,
}

impl ParseTreeCtor for Move {
    const LEXEME: &'static str = "move";
    fn parse(term: &OtpErlangTerm) -> ParseTree {
        let move_tuple = try_parse!(term, expect_tuple(term, 3));

        let rvalue = Box::new(parse(&move_tuple[1]));
        let lvalue = Box::new(parse(&move_tuple[2]));
        let move_token = Move { rvalue, lvalue };

        ParseTree::Move(move_token)
    }

    fn dump(&self) -> String {
        let tr1 = self.lvalue.dump();
        let tr2 = self.rvalue.dump();
        format!("{} = {}", tr1, tr2)
    }
}

pub struct CallExt {
    pub arity: i32,
    pub func: Box<ParseTree>,
}

impl ParseTreeCtor for CallExt {
    const LEXEME: &'static str = "call_ext";
    fn parse(term: &OtpErlangTerm) -> ParseTree {
        let callext_tuple = try_parse!(term, expect_tuple(term, 3));
        let arity = try_parse!(term, expect_integer(&callext_tuple[1]));
        let func = Box::new(parse(&callext_tuple[2]));

        let call_ext = CallExt { arity, func };

        ParseTree::CallExt(call_ext)
    }

    fn dump(&self) -> String {
        format!("X0 = {}", self.func.dump())
    }
}

pub struct ExtFunc {
    pub module: String,
    pub name: String,
    pub arity: i32,
}

impl ParseTreeCtor for ExtFunc {
    const LEXEME: &'static str = "extfunc";
    fn parse(term: &OtpErlangTerm) -> ParseTree {
        let extfunc_tuple = try_parse!(term, expect_tuple(term, 4));

        let module = try_parse!(term, atomutf8_to_string(&extfunc_tuple[1]));
        let name = try_parse!(term, atomutf8_to_string(&extfunc_tuple[2]));

        let arity = try_parse!(term, expect_integer(&extfunc_tuple[3]));

        let ext_func = ExtFunc {
            module,
            name,
            arity,
        };

        ParseTree::ExtFunc(ext_func)
    }

    fn dump(&self) -> String {
        let args = (0..self.arity)
            .map(|x| format!("X{}", x))
            .collect::<Vec<_>>()
            .join(", ");
        let tr = format!("{}:{}({})", self.module, self.name, args);
        tr
    }
}

pub struct FLabel {
    pub num: i32,
}

impl ParseTreeCtor for FLabel {
    const LEXEME: &'static str = "f";
    fn parse(term: &OtpErlangTerm) -> ParseTree {
        let flabel_tuple = try_parse!(term, expect_tuple(term, 2));
        let num = try_parse!(term, expect_integer(&flabel_tuple[1]));

        let flabel = FLabel { num };
        ParseTree::FLabel(flabel)
    }

    fn dump(&self) -> String {
        format!("label{}", self.num)
    }
}

pub struct GcBif {
    pub name: String,
    pub fallback: Box<ParseTree>,
    pub arity: i32,
    pub args: Vec<ParseTree>,
    pub store: Box<ParseTree>,
}

impl ParseTreeCtor for GcBif {
    const LEXEME: &'static str = "gc_bif";
    fn parse(term: &OtpErlangTerm) -> ParseTree {
        let gcbif_tuple = try_parse!(term, expect_tuple(term, 6));
        let name = try_parse!(term, atomutf8_to_string(&gcbif_tuple[1]));

        let fallback = Box::new(parse(&gcbif_tuple[2]));

        let arity = try_parse!(term, expect_integer(&gcbif_tuple[3]));

        let args_terms = try_parse!(term, expect_list(&gcbif_tuple[4]));

        let mut args: Vec<ParseTree> = Vec::with_capacity(args_terms.len());
        for arg_term in args_terms {
            let token = parse(arg_term);
            args.push(token);
        }

        let store = Box::new(parse(&gcbif_tuple[5]));

        let gcbif = GcBif {
            name,
            fallback,
            arity,
            args,
            store,
        };

        ParseTree::GcBif(gcbif)
    }

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

pub struct Literal {
    pub s: String,
}

impl ParseTreeCtor for Literal {
    const LEXEME: &'static str = "literal";
    fn parse(term: &OtpErlangTerm) -> ParseTree {
        let literal_tuple = try_parse!(term, expect_tuple(term, 2));
        let s = try_parse!(term, expect_string(&literal_tuple[1]));

        let literal = Literal { s };
        ParseTree::Literal(literal)
    }

    fn dump(&self) -> String {
        format!("{:?}", self.s)
    }
}

pub struct CallExtLast {
    pub arity: i32,
    pub func: Box<ParseTree>,
}

impl ParseTreeCtor for CallExtLast {
    const LEXEME: &'static str = "call_ext_last";
    fn parse(term: &OtpErlangTerm) -> ParseTree {
        let callext_tuple = try_parse!(term, expect_tuple(term, 4));
        let arity = try_parse!(term, expect_integer(&callext_tuple[1]));
        let func = Box::new(parse(&callext_tuple[2]));

        let call_ext_last = CallExtLast { arity, func };

        ParseTree::CallExtLast(call_ext_last)
    }

    fn dump(&self) -> String {
        format!("return {}", self.func.dump())
    }
}

pub struct CallExtOnly {
    pub arity: i32,
    pub func: Box<ParseTree>,
}

impl ParseTreeCtor for CallExtOnly {
    const LEXEME: &'static str = "call_ext_only";
    fn parse(term: &OtpErlangTerm) -> ParseTree {
        let callext_tuple = try_parse!(term, expect_tuple(term, 3));

        let arity = try_parse!(term, expect_integer(&callext_tuple[1]));
        let func = Box::new(parse(&callext_tuple[2]));

        let call_ext_only = CallExtOnly { arity, func };

        ParseTree::CallExtOnly(call_ext_only)
    }

    fn dump(&self) -> String {
        format!("return {}", self.func.dump())
    }
}

pub struct Integer {
    pub num: i32,
}

impl ParseTreeCtor for Integer {
    const LEXEME: &'static str = "integer";
    fn parse(term: &OtpErlangTerm) -> ParseTree {
        let int_tuple = try_parse!(term, expect_tuple(term, 2));
        let num = try_parse!(term, expect_integer(&int_tuple[1]));
        let integer = Integer { num };
        ParseTree::Integer(integer)
    }

    fn dump(&self) -> String {
        format!("{}", self.num)
    }
}

pub struct PutList {
    pub head: Box<ParseTree>,
    pub tail: Box<ParseTree>,
    pub store: Box<ParseTree>,
}

impl ParseTreeCtor for PutList {
    const LEXEME: &'static str = "put_list";
    fn parse(term: &OtpErlangTerm) -> ParseTree {
        let putlist_tuple = try_parse!(term, expect_tuple(term, 4));
        let head = Box::new(parse(&putlist_tuple[1]));
        let tail = Box::new(parse(&putlist_tuple[2]));

        let store = Box::new(parse(&putlist_tuple[3]));

        let put_list = PutList { head, tail, store };

        ParseTree::PutList(put_list)
    }

    fn dump(&self) -> String {
        let head_tr = self.head.dump();
        let store_tr = self.store.dump();
        if let ParseTree::Nil(_) = *self.tail {
            format!("{} = [{}]", store_tr, head_tr)
        } else {
            let tail_tr = self.tail.dump();
            format!("{} = [{} | {}]", store_tr, head_tr, tail_tr)
        }
    }
}

pub struct Nil;

impl ParseTreeCtor for Nil {
    const LEXEME: &'static str = "nil";
    fn parse(_: &OtpErlangTerm) -> ParseTree {
        ParseTree::Nil(Nil {})
    }

    fn dump(&self) -> String {
        "nil".into()
    }
}

pub struct Atom {
    pub name: String,
}

impl ParseTreeCtor for Atom {
    const LEXEME: &'static str = "atom";
    fn parse(term: &OtpErlangTerm) -> ParseTree {
        let atom_tuple = try_parse!(term, expect_tuple(term, 2));
        let name = try_parse!(term, atomutf8_to_string(&atom_tuple[1]));
        let atom = Atom { name };
        ParseTree::Atom(atom)
    }

    fn dump(&self) -> String {
        self.name.clone()
    }
}

pub struct TInteger {
    pub num1: i32,
    pub num2: i32,
}

impl ParseTreeCtor for TInteger {
    const LEXEME: &'static str = "t_integer";
    fn parse(term: &OtpErlangTerm) -> ParseTree {
        let tint_tuple = try_parse!(term, expect_tuple(term, 2));
        let num_range = try_parse!(term, expect_tuple(&tint_tuple[1], 2));
        let num1 = try_parse!(term, expect_integer(&num_range[0]));
        let num2 = try_parse!(term, expect_integer(&num_range[1]));
        let tint = TInteger { num1, num2 };
        ParseTree::TInteger(tint)
    }

    fn dump(&self) -> String {
        format!("{} - {}", self.num1, self.num2)
    }
}

pub struct Tr {
    pub reg: Box<ParseTree>,
    pub ty: Box<ParseTree>,
}

impl ParseTreeCtor for Tr {
    const LEXEME: &'static str = "tr";
    fn parse(term: &OtpErlangTerm) -> ParseTree {
        let tr_tuple = try_parse!(term, expect_tuple(term, 3));

        let reg = Box::new(parse(&tr_tuple[1]));
        let ty = Box::new(parse(&tr_tuple[2]));

        let tr = Tr { reg, ty };
        ParseTree::Tr(tr)
    }

    fn dump(&self) -> String {
        let reg_tr = self.reg.dump();
        format!("{}", reg_tr)
    }
}

pub struct Test {
    pub comp: Box<ParseTree>,
    pub fail: Box<ParseTree>,
    pub args: Vec<ParseTree>,
}

impl ParseTreeCtor for Test {
    const LEXEME: &'static str = "test";
    fn parse(term: &OtpErlangTerm) -> ParseTree {
        let test_tuple = try_parse!(term, expect_tuple(term, 4));

        let comp = Box::new(parse(&test_tuple[1]));
        let fail = Box::new(parse(&test_tuple[2]));
        let args_terms = try_parse!(term, expect_list(&test_tuple[3]));

        let mut args: Vec<ParseTree> = Vec::with_capacity(args_terms.len());
        for arg_term in args_terms {
            let parsed_arg = parse(arg_term);
            args.push(parsed_arg);
        }

        let test = Test { comp, fail, args };
        ParseTree::Test(test)
    }

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

pub struct Line {
    pub num: i32,
}

impl ParseTreeCtor for Line {
    const LEXEME: &'static str = "line";
    fn parse(term: &OtpErlangTerm) -> ParseTree {
        let tuple = try_parse!(term, expect_tuple(term, 2));
        let num = try_parse!(term, expect_integer(&tuple[1]));

        let line = Line { num };
        ParseTree::Line(line)
    }

    fn dump(&self) -> String {
        format!("# line {}", self.num)
    }
}

lexeme_as_token! {IsGe, "is_ge" }
lexeme_as_token! {IsEqExact, "is_eq_exact"}

info_token! {FuncInfo, "func_info" }
info_token! {Allocate, "allocate"}
info_token! {InitYRegs, "init_yregs"}
info_token! {TestHeap, "test_heap"}
