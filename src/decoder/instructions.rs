use crate::translation::{Instr, Token};
use crate::decoder::util::atomutf8_to_string;
use erlang::OtpErlangTerm;
use once_cell::sync::Lazy;
use std::collections::HashMap;

static INSTR_REGISTRY_LIST: [(&'static str, InstrCtor); 0] = [];

type InstrCtor = fn(&OtpErlangTerm) -> Box<dyn Instr>;

static INSTR_REGISTRY: Lazy<HashMap<&'static str, InstrCtor>> =
    Lazy::new(|| HashMap::from(INSTR_REGISTRY_LIST));

macro_rules! extrtuple {
    ($tuple_name:ident, $term:ident) => {
        let OtpErlangTerm::OtpErlangTuple($tuple_name) = $term else {
            return Err("Instr term must be a tuple");
        };
    };
}

macro_rules! new_instr {
    ($type:ty, $is_ctrl:literal, $func:expr) => {
        impl Token for $type {
            fn decode(term: &OtpErlangTerm) -> Result<$type, &'static str> {
                $func(term)
            }
        }

        impl Instr for $type {
            fn is_ctrl(&self) -> bool {
                $is_ctrl
            }
        }
    };
}

pub fn construct_instr(term: &OtpErlangTerm) -> Result<Box<dyn Instr>, &'static str> {
    extrtuple!(instr_tuple, term);

    let OtpErlangTerm::OtpErlangAtomUTF8(name_atom) = &instr_tuple[0] else {
        return Err("Instr tuple must contain it's name as 1st element");
    };
    let name = atomutf8_to_string(name_atom)?;

    if !INSTR_REGISTRY.contains_key(name.as_str()) {
        return Ok(Box::new(Unresolved {}));
    }

    Ok(INSTR_REGISTRY[name.as_str()](term))
}

struct Unresolved {}
new_instr!(Unresolved, false, |_| { Ok(Unresolved {}) });
