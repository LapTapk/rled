use erlang::OtpErlangTerm;

#[derive(Debug)]
pub enum LexemeError {
    ExpectedAtomOrTuple { found: OtpErlangTerm },
    EmptyTuple,
    Unsupported { found: OtpErlangTerm },
}
