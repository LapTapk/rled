use crate::core::parse::errors::LexemeError;
use crate::core::parse::parse::*;
use crate::core::parse::util::atomutf8_to_string;
use crate::core::syntax::*;
use erlang::OtpErlangTerm;

pub trait ParseLexeme {
    const LEXEME: &'static str;
}

pub type ParseFn = fn(&OtpErlangTerm) -> SyntaxNode;

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

macro_rules! construct_dispatch {
    (
        $( $node:ident ),* $(,)?
    ) => {

        pub fn parse_dispatch(lexeme: &str) -> ParseFn {
            match lexeme {
            $(
                <$node as ParseLexeme>::LEXEME => <$node as ParseNode>::parse,
            )*
                _ => Unparsed::parse,
            }
        }

    };
}

macro_rules! lexeme {
    ($node:ident, $lexeme:literal) => {
        impl ParseLexeme for $node {
            const LEXEME: &'static str = $lexeme;
        }
    };
}

construct_dispatch!(
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

lexeme!(Module, "beam_file");
lexeme!(Func, "function");
lexeme!(Move, "move");
lexeme!(CallExt, "call_ext");
lexeme!(GcBif, "gc_bif");
lexeme!(PutList, "put_list");
lexeme!(Tr, "tr");
lexeme!(Test, "test");
lexeme!(CallExtLast, "call_ext_last");
lexeme!(CallExtOnly, "call_ext_only");
lexeme!(Label, "label");
lexeme!(XReg, "x");
lexeme!(YReg, "y");
lexeme!(ExtFunc, "extfunc");
lexeme!(FLabel, "f");
lexeme!(Literal, "literal");
lexeme!(Integer, "integer");
lexeme!(Nil, "nil");
lexeme!(Atom, "atom");
lexeme!(TInteger, "t_integer");
lexeme!(Line, "line");
lexeme!(FuncInfo, "func_info");
lexeme!(Allocate, "allocate");
lexeme!(TestHeap, "test_heap");
lexeme!(InitYRegs, "init_yregs");
lexeme!(IsGe, "is_ge");
lexeme!(IsEqExact, "is_eq_exact");
