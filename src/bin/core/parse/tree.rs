use crate::core::syntax::SyntaxNode;
use erlang::OtpErlangTerm;

pub trait ParseLexeme {
    const LEXEME: &'static str;
}

pub trait ParseTree {
    fn parse(term: &OtpErlangTerm) -> SyntaxNode;
}
