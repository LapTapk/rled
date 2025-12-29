use crate::core::parse::grammar::*;
use crate::core::parse::lexeme::{get_lexeme, parse_dispatch};
use crate::core::parse::util::atomutf8_to_string;
use crate::core::syntax::*;
use erlang::OtpErlangTerm;

pub trait ParseNode {
    fn parse(term: &OtpErlangTerm) -> SyntaxNode;
}

impl ParseNode for Unparsed {
    fn parse(term: &OtpErlangTerm) -> SyntaxNode {
        SyntaxNode::Unparsed(Unparsed {
            term: term.clone(),
            reason: "SyntaxNode unregistered".into(),
        })
    }
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

macro_rules! empty_node {
    {$type:ident} => {
        impl ParseNode for $type {
            fn parse(_: &OtpErlangTerm) -> SyntaxNode {
                SyntaxNode::$type($type{})
            }
        }
    };
}

impl ParseNode for SyntaxNode {
    fn parse(term: &OtpErlangTerm) -> SyntaxNode {
        let lexeme = try_parse!(term, get_lexeme(term));
        parse_dispatch(lexeme.as_str())(term)
    }
}


impl ParseNode for Module {
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

impl ParseNode for Func {
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

impl ParseNode for Label {
    fn parse(term: &OtpErlangTerm) -> SyntaxNode {
        let label_tuple = try_parse!(term, expect_tuple(term, 2));

        let OtpErlangTerm::OtpErlangInteger(num) = &label_tuple[1] else {
            return Unparsed::new(term, "Label must have integer as it's 2nd element");
        };

        let label = Label { num: *num };
        SyntaxNode::Label(label)
    }
}

impl ParseNode for XReg {
    fn parse(term: &OtpErlangTerm) -> SyntaxNode {
        let xreg_tuple = try_parse!(term, expect_tuple(term, 2));
        let num = try_parse!(term, expect_integer(&xreg_tuple[1]));

        let xreg = XReg { num };
        SyntaxNode::XReg(xreg)
    }
}

impl ParseNode for YReg {
    fn parse(term: &OtpErlangTerm) -> SyntaxNode {
        let yreg_tuple = try_parse!(term, expect_tuple(term, 2));
        let num = try_parse!(term, expect_integer(&yreg_tuple[1]));

        let yreg = YReg { num };
        SyntaxNode::YReg(yreg)
    }
}

impl ParseNode for Move {
    fn parse(term: &OtpErlangTerm) -> SyntaxNode {
        let move_tuple = try_parse!(term, expect_tuple(term, 3));

        let rvalue = Box::new(SyntaxNode::parse(&move_tuple[1]));
        let lvalue = Box::new(SyntaxNode::parse(&move_tuple[2]));
        let move_token = Move { rvalue, lvalue };

        SyntaxNode::Move(move_token)
    }
}

impl ParseNode for CallExt {
    fn parse(term: &OtpErlangTerm) -> SyntaxNode {
        let callext_tuple = try_parse!(term, expect_tuple(term, 3));
        let arity = try_parse!(term, expect_integer(&callext_tuple[1]));
        let func = Box::new(SyntaxNode::parse(&callext_tuple[2]));

        let call_ext = CallExt { arity, func };

        SyntaxNode::CallExt(call_ext)
    }
}

impl ParseNode for ExtFunc {
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

impl ParseNode for FLabel {
    fn parse(term: &OtpErlangTerm) -> SyntaxNode {
        let flabel_tuple = try_parse!(term, expect_tuple(term, 2));
        let num = try_parse!(term, expect_integer(&flabel_tuple[1]));

        let flabel = FLabel { num };
        SyntaxNode::FLabel(flabel)
    }
}

impl ParseNode for GcBif {
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

impl ParseNode for Literal {
    fn parse(term: &OtpErlangTerm) -> SyntaxNode {
        let literal_tuple = try_parse!(term, expect_tuple(term, 2));
        let s = try_parse!(term, expect_string(&literal_tuple[1]));

        let literal = Literal { s };
        SyntaxNode::Literal(literal)
    }
}

impl ParseNode for CallExtLast {
    fn parse(term: &OtpErlangTerm) -> SyntaxNode {
        let callext_tuple = try_parse!(term, expect_tuple(term, 4));
        let arity = try_parse!(term, expect_integer(&callext_tuple[1]));
        let func = Box::new(SyntaxNode::parse(&callext_tuple[2]));

        let call_ext_last = CallExtLast { arity, func };

        SyntaxNode::CallExtLast(call_ext_last)
    }
}

impl ParseNode for CallExtOnly {
    fn parse(term: &OtpErlangTerm) -> SyntaxNode {
        let callext_tuple = try_parse!(term, expect_tuple(term, 3));

        let arity = try_parse!(term, expect_integer(&callext_tuple[1]));
        let func = Box::new(SyntaxNode::parse(&callext_tuple[2]));

        let call_ext_only = CallExtOnly { arity, func };

        SyntaxNode::CallExtOnly(call_ext_only)
    }
}

impl ParseNode for Integer {
    fn parse(term: &OtpErlangTerm) -> SyntaxNode {
        let int_tuple = try_parse!(term, expect_tuple(term, 2));
        let num = try_parse!(term, expect_integer(&int_tuple[1]));
        let integer = Integer { num };
        SyntaxNode::Integer(integer)
    }
}

impl ParseNode for PutList {
    fn parse(term: &OtpErlangTerm) -> SyntaxNode {
        let putlist_tuple = try_parse!(term, expect_tuple(term, 4));
        let head = Box::new(SyntaxNode::parse(&putlist_tuple[1]));
        let tail = Box::new(SyntaxNode::parse(&putlist_tuple[2]));

        let store = Box::new(SyntaxNode::parse(&putlist_tuple[3]));

        let put_list = PutList { head, tail, store };

        SyntaxNode::PutList(put_list)
    }
}

impl ParseNode for Nil {
    fn parse(_: &OtpErlangTerm) -> SyntaxNode {
        SyntaxNode::Nil(Nil {})
    }
}

impl ParseNode for Atom {
    fn parse(term: &OtpErlangTerm) -> SyntaxNode {
        let atom_tuple = try_parse!(term, expect_tuple(term, 2));
        let name = try_parse!(term, atomutf8_to_string(&atom_tuple[1]));
        let atom = Atom { name };
        SyntaxNode::Atom(atom)
    }
}

impl ParseNode for TInteger {
    fn parse(term: &OtpErlangTerm) -> SyntaxNode {
        let tint_tuple = try_parse!(term, expect_tuple(term, 2));
        let num_range = try_parse!(term, expect_tuple(&tint_tuple[1], 2));
        let num1 = try_parse!(term, expect_integer(&num_range[0]));
        let num2 = try_parse!(term, expect_integer(&num_range[1]));
        let tint = TInteger { num1, num2 };
        SyntaxNode::TInteger(tint)
    }
}

impl ParseNode for Tr {
    fn parse(term: &OtpErlangTerm) -> SyntaxNode {
        let tr_tuple = try_parse!(term, expect_tuple(term, 3));

        let reg = Box::new(SyntaxNode::parse(&tr_tuple[1]));
        let ty = Box::new(SyntaxNode::parse(&tr_tuple[2]));

        let tr = Tr { reg, ty };
        SyntaxNode::Tr(tr)
    }
}

impl ParseNode for Test {
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

impl ParseNode for Line {
    fn parse(term: &OtpErlangTerm) -> SyntaxNode {
        let tuple = try_parse!(term, expect_tuple(term, 2));
        let num = try_parse!(term, expect_integer(&tuple[1]));

        let line = Line { num };
        SyntaxNode::Line(line)
    }
}

empty_node! {IsGe}
empty_node! {IsEqExact}
empty_node! {FuncInfo}
empty_node! {Allocate}
empty_node! {InitYRegs}
empty_node! {TestHeap}
