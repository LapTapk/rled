use crate::core::parse::nodes;
use crate::core::parse::tree::ParseTree;
use crate::core::syntax::{Module, SyntaxNode, SyntaxTree};
use erlang::OtpErlangTerm;

pub fn parse_and_dump(term: &OtpErlangTerm) -> String {
    Module::parse(term).dump()
}

impl ParseTree for SyntaxNode {
    fn parse(term: &OtpErlangTerm) -> SyntaxNode {
        nodes::parse_node(term)
    }
}
