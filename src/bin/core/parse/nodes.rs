use crate::core::parse::errors::LexemeError;
use crate::core::parse::grammar::{expect_integer, expect_list, expect_string, expect_tuple};
use crate::core::parse::tree::{ParseLexeme, ParseTree};
use crate::core::parse::util::atomutf8_to_string;
use crate::core::syntax::*;
use erlang::OtpErlangTerm;

type ParseFn = fn(&OtpErlangTerm) -> SyntaxNode;

pub(crate) fn parse_node(term: &OtpErlangTerm) -> SyntaxNode {
    let lexeme = try_parse!(term, get_lexeme(term));
    parse_dispatch(lexeme.as_str())(term)
}

fn get_lexeme(term: &OtpErlangTerm) -> Result<String, LexemeError> {
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

macro_rules! try_parse {
    ($term:expr , $e:expr) => {
        match $e {
            Ok(ok) => ok,
            Err(e) => {
                return SyntaxNode::Unparsed(Unparsed {
                    term: $term.clone(),
                    reason: format!("{:?}", e),
                });
            }
        }
    };
}

macro_rules! construct_helpers {
    (
        $( $node:ident ),* $(,)?
    ) => {

        fn parse_dispatch(lexeme: &str) -> ParseFn {
            match lexeme {
            $(
                <$node as ParseLexeme>::LEXEME => <$node as ParseTree>::parse,
            )*
                _ => Unparsed::parse,
            }
        }
    };
}

construct_helpers!(
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
    IsEqExact
);

macro_rules! empty_node {
    {$type:ident, $lexeme:literal} => {
        impl ParseLexeme for $type {
            const LEXEME: &'static str = $lexeme;
        }

        impl ParseTree for $type {
            fn parse(_: &OtpErlangTerm) -> SyntaxNode {
                SyntaxNode::$type($type{})
            }
        }
    };
}

impl ParseTree for Unparsed {
    fn parse(term: &OtpErlangTerm) -> SyntaxNode {
        SyntaxNode::Unparsed(Unparsed {
            term: term.clone(),
            reason: "SyntaxNode unregistered".into(),
        })
    }
}

impl ParseLexeme for Module {
    const LEXEME: &'static str = "beam_file";
}

impl ParseTree for Module {
    fn parse(term: &OtpErlangTerm) -> SyntaxNode {
        let module = try_parse!(term, expect_tuple(term, 6));
        let name = try_parse!(term, atomutf8_to_string(&module[1]));
        let funcs_beam = try_parse!(term, expect_list(&module[5]));

        let mut funcs: Vec<SyntaxNode> = Vec::with_capacity(funcs_beam.len());
        for func_beam in funcs_beam {
            let parsed_func = SyntaxNode::parse(func_beam);
            funcs.push(parsed_func);
        }

        let module = Module { name, funcs };
        SyntaxNode::Module(module)
    }
}

impl ParseLexeme for Func {
    const LEXEME: &'static str = "function";
}

impl ParseTree for Func {
    fn parse(term: &OtpErlangTerm) -> SyntaxNode {
        let func_tuple = try_parse!(term, expect_tuple(term, 5));
        let name = try_parse!(term, atomutf8_to_string(&func_tuple[1]));
        let arity = try_parse!(term, expect_integer(&func_tuple[2]));
        let label = try_parse!(term, expect_integer(&func_tuple[3]));
        let instrs_list = try_parse!(term, expect_list(&func_tuple[4]));

        let mut instrs: Vec<SyntaxNode> = Vec::with_capacity(instrs_list.len());
        for instr_term in instrs_list {
            let parsed_instr = SyntaxNode::parse(instr_term);
            instrs.push(parsed_instr);
        }

        let func = Func {
            name,
            arity,
            label,
            instrs,
        };
        SyntaxNode::Func(func)
    }
}

impl ParseLexeme for Label {
    const LEXEME: &'static str = "label";
}

impl ParseTree for Label {
    fn parse(term: &OtpErlangTerm) -> SyntaxNode {
        let label_tuple = try_parse!(term, expect_tuple(term, 2));

        let OtpErlangTerm::OtpErlangInteger(num) = &label_tuple[1] else {
            return Unparsed::new(term, "Label must have integer as it's 2nd element");
        };

        let label = Label { num: *num };
        SyntaxNode::Label(label)
    }
}

impl ParseLexeme for XReg {
    const LEXEME: &'static str = "x";
}

impl ParseTree for XReg {
    fn parse(term: &OtpErlangTerm) -> SyntaxNode {
        let xreg_tuple = try_parse!(term, expect_tuple(term, 2));
        let num = try_parse!(term, expect_integer(&xreg_tuple[1]));

        let xreg = XReg { num };
        SyntaxNode::XReg(xreg)
    }
}

impl ParseLexeme for YReg {
    const LEXEME: &'static str = "y";
}

impl ParseTree for YReg {
    fn parse(term: &OtpErlangTerm) -> SyntaxNode {
        let yreg_tuple = try_parse!(term, expect_tuple(term, 2));
        let num = try_parse!(term, expect_integer(&yreg_tuple[1]));

        let yreg = YReg { num };
        SyntaxNode::YReg(yreg)
    }
}

impl ParseLexeme for Move {
    const LEXEME: &'static str = "move";
}

impl ParseTree for Move {
    fn parse(term: &OtpErlangTerm) -> SyntaxNode {
        let move_tuple = try_parse!(term, expect_tuple(term, 3));

        let rvalue = Box::new(SyntaxNode::parse(&move_tuple[1]));
        let lvalue = Box::new(SyntaxNode::parse(&move_tuple[2]));
        let move_token = Move { rvalue, lvalue };

        SyntaxNode::Move(move_token)
    }
}

impl ParseLexeme for CallExt {
    const LEXEME: &'static str = "call_ext";
}

impl ParseTree for CallExt {
    fn parse(term: &OtpErlangTerm) -> SyntaxNode {
        let callext_tuple = try_parse!(term, expect_tuple(term, 3));
        let arity = try_parse!(term, expect_integer(&callext_tuple[1]));
        let func = Box::new(SyntaxNode::parse(&callext_tuple[2]));

        let call_ext = CallExt { arity, func };

        SyntaxNode::CallExt(call_ext)
    }
}

impl ParseLexeme for ExtFunc {
    const LEXEME: &'static str = "extfunc";
}

impl ParseTree for ExtFunc {
    fn parse(term: &OtpErlangTerm) -> SyntaxNode {
        let extfunc_tuple = try_parse!(term, expect_tuple(term, 4));

        let module = try_parse!(term, atomutf8_to_string(&extfunc_tuple[1]));
        let name = try_parse!(term, atomutf8_to_string(&extfunc_tuple[2]));

        let arity = try_parse!(term, expect_integer(&extfunc_tuple[3]));

        let ext_func = ExtFunc {
            module,
            name,
            arity,
        };

        SyntaxNode::ExtFunc(ext_func)
    }
}

impl ParseLexeme for FLabel {
    const LEXEME: &'static str = "f";
}

impl ParseTree for FLabel {
    fn parse(term: &OtpErlangTerm) -> SyntaxNode {
        let flabel_tuple = try_parse!(term, expect_tuple(term, 2));
        let num = try_parse!(term, expect_integer(&flabel_tuple[1]));

        let flabel = FLabel { num };
        SyntaxNode::FLabel(flabel)
    }
}

impl ParseLexeme for GcBif {
    const LEXEME: &'static str = "gc_bif";
}

impl ParseTree for GcBif {
    fn parse(term: &OtpErlangTerm) -> SyntaxNode {
        let gcbif_tuple = try_parse!(term, expect_tuple(term, 6));
        let name = try_parse!(term, atomutf8_to_string(&gcbif_tuple[1]));

        let fallback = Box::new(SyntaxNode::parse(&gcbif_tuple[2]));

        let arity = try_parse!(term, expect_integer(&gcbif_tuple[3]));

        let args_terms = try_parse!(term, expect_list(&gcbif_tuple[4]));

        let mut args: Vec<SyntaxNode> = Vec::with_capacity(args_terms.len());
        for arg_term in args_terms {
            let token = SyntaxNode::parse(arg_term);
            args.push(token);
        }

        let store = Box::new(SyntaxNode::parse(&gcbif_tuple[5]));

        let gcbif = GcBif {
            name,
            fallback,
            arity,
            args,
            store,
        };

        SyntaxNode::GcBif(gcbif)
    }
}

impl ParseLexeme for Literal {
    const LEXEME: &'static str = "literal";
}

impl ParseTree for Literal {
    fn parse(term: &OtpErlangTerm) -> SyntaxNode {
        let literal_tuple = try_parse!(term, expect_tuple(term, 2));
        let s = try_parse!(term, expect_string(&literal_tuple[1]));

        let literal = Literal { s };
        SyntaxNode::Literal(literal)
    }
}

impl ParseLexeme for CallExtLast {
    const LEXEME: &'static str = "call_ext_last";
}

impl ParseTree for CallExtLast {
    fn parse(term: &OtpErlangTerm) -> SyntaxNode {
        let callext_tuple = try_parse!(term, expect_tuple(term, 4));
        let arity = try_parse!(term, expect_integer(&callext_tuple[1]));
        let func = Box::new(SyntaxNode::parse(&callext_tuple[2]));

        let call_ext_last = CallExtLast { arity, func };

        SyntaxNode::CallExtLast(call_ext_last)
    }
}

impl ParseLexeme for CallExtOnly {
    const LEXEME: &'static str = "call_ext_only";
}

impl ParseTree for CallExtOnly {
    fn parse(term: &OtpErlangTerm) -> SyntaxNode {
        let callext_tuple = try_parse!(term, expect_tuple(term, 3));

        let arity = try_parse!(term, expect_integer(&callext_tuple[1]));
        let func = Box::new(SyntaxNode::parse(&callext_tuple[2]));

        let call_ext_only = CallExtOnly { arity, func };

        SyntaxNode::CallExtOnly(call_ext_only)
    }
}

impl ParseLexeme for Integer {
    const LEXEME: &'static str = "integer";
}

impl ParseTree for Integer {
    fn parse(term: &OtpErlangTerm) -> SyntaxNode {
        let int_tuple = try_parse!(term, expect_tuple(term, 2));
        let num = try_parse!(term, expect_integer(&int_tuple[1]));
        let integer = Integer { num };
        SyntaxNode::Integer(integer)
    }
}

impl ParseLexeme for PutList {
    const LEXEME: &'static str = "put_list";
}

impl ParseTree for PutList {
    fn parse(term: &OtpErlangTerm) -> SyntaxNode {
        let putlist_tuple = try_parse!(term, expect_tuple(term, 4));
        let head = Box::new(SyntaxNode::parse(&putlist_tuple[1]));
        let tail = Box::new(SyntaxNode::parse(&putlist_tuple[2]));

        let store = Box::new(SyntaxNode::parse(&putlist_tuple[3]));

        let put_list = PutList { head, tail, store };

        SyntaxNode::PutList(put_list)
    }
}

impl ParseLexeme for Nil {
    const LEXEME: &'static str = "nil";
}

impl ParseTree for Nil {
    fn parse(_: &OtpErlangTerm) -> SyntaxNode {
        SyntaxNode::Nil(Nil {})
    }
}

impl ParseLexeme for Atom {
    const LEXEME: &'static str = "atom";
}

impl ParseTree for Atom {
    fn parse(term: &OtpErlangTerm) -> SyntaxNode {
        let atom_tuple = try_parse!(term, expect_tuple(term, 2));
        let name = try_parse!(term, atomutf8_to_string(&atom_tuple[1]));
        let atom = Atom { name };
        SyntaxNode::Atom(atom)
    }
}

impl ParseLexeme for TInteger {
    const LEXEME: &'static str = "t_integer";
}

impl ParseTree for TInteger {
    fn parse(term: &OtpErlangTerm) -> SyntaxNode {
        let tint_tuple = try_parse!(term, expect_tuple(term, 2));
        let num_range = try_parse!(term, expect_tuple(&tint_tuple[1], 2));
        let num1 = try_parse!(term, expect_integer(&num_range[0]));
        let num2 = try_parse!(term, expect_integer(&num_range[1]));
        let tint = TInteger { num1, num2 };
        SyntaxNode::TInteger(tint)
    }
}

impl ParseLexeme for Tr {
    const LEXEME: &'static str = "tr";
}

impl ParseTree for Tr {
    fn parse(term: &OtpErlangTerm) -> SyntaxNode {
        let tr_tuple = try_parse!(term, expect_tuple(term, 3));

        let reg = Box::new(SyntaxNode::parse(&tr_tuple[1]));
        let ty = Box::new(SyntaxNode::parse(&tr_tuple[2]));

        let tr = Tr { reg, ty };
        SyntaxNode::Tr(tr)
    }
}

impl ParseLexeme for Test {
    const LEXEME: &'static str = "test";
}

impl ParseTree for Test {
    fn parse(term: &OtpErlangTerm) -> SyntaxNode {
        let test_tuple = try_parse!(term, expect_tuple(term, 4));

        let comp = Box::new(SyntaxNode::parse(&test_tuple[1]));
        let fail = Box::new(SyntaxNode::parse(&test_tuple[2]));
        let args_terms = try_parse!(term, expect_list(&test_tuple[3]));

        let mut args: Vec<SyntaxNode> = Vec::with_capacity(args_terms.len());
        for arg_term in args_terms {
            let parsed_arg = SyntaxNode::parse(arg_term);
            args.push(parsed_arg);
        }

        let test = Test { comp, fail, args };
        SyntaxNode::Test(test)
    }
}

impl ParseLexeme for Line {
    const LEXEME: &'static str = "line";
}

impl ParseTree for Line {
    fn parse(term: &OtpErlangTerm) -> SyntaxNode {
        let tuple = try_parse!(term, expect_tuple(term, 2));
        let num = try_parse!(term, expect_integer(&tuple[1]));

        let line = Line { num };
        SyntaxNode::Line(line)
    }
}

empty_node! {IsGe, "is_ge" }
empty_node! {IsEqExact, "is_eq_exact"}
empty_node! {FuncInfo, "func_info" }
empty_node! {Allocate, "allocate"}
empty_node! {InitYRegs, "init_yregs"}
empty_node! {TestHeap, "test_heap"}
