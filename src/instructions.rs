use crate::util::atomutf8_to_string;
use crate::translation::*;
use erlang::OtpErlangTerm;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::collections::LinkedList;

static INSTR_REGISTRY_LIST: [(&'static str, InstrCtor); 0] = [];

type InstrCtor =
    for<'a> fn(&OtpErlangTerm, slist: SymbolList<'a>) -> Result<Parsed<'a>, &'static str>;

fn parse_generic<'a, T>(
    term: &OtpErlangTerm,
    slist: SymbolList<'a>,
) -> Result<Parsed<'a>, &'static str>
where
    T: Token,
{
    T::parse(term, slist)
}

static INSTR_REGISTRY: Lazy<HashMap<&'static str, InstrCtor>> =
    Lazy::new(|| HashMap::from(INSTR_REGISTRY_LIST));

macro_rules! extrtuple {
    ($tuple_name:ident, $term:ident) => {
        let OtpErlangTerm::OtpErlangTuple($tuple_name) = $term else {
            return Err("Instr term must be a tuple");
        };
    };
}

pub fn parse_instr<'a>(
    term: &OtpErlangTerm,
    slist: SymbolList<'a>,
) -> Result<Parsed<'a>, &'static str> {
    extrtuple!(instr_tuple, term);

    let OtpErlangTerm::OtpErlangAtomUTF8(name_atom) = &instr_tuple[0] else {
        return Err("Instr tuple must contain it's name as 1st element");
    };
    let name = atomutf8_to_string(name_atom)?;

    if !INSTR_REGISTRY.contains_key(name.as_str()) {
        return parse_generic::<Unresolved>(term, slist);
    }

    INSTR_REGISTRY[name.as_str()](term, slist)
}

struct Unresolved {}

impl Token for Unresolved {
    fn parse<'a>(term: &OtpErlangTerm, slist: SymbolList<'a>) -> Result<Parsed<'a>, &'static str> {
        Ok(Parsed {
            token: Box::new(Unresolved {}),
            slist: LinkedList::new(),
        })
    }
}
