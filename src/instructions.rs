use crate::translation::*;
use crate::util::atomutf8_to_string;
use erlang::OtpErlangTerm;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::collections::LinkedList;

static TOKEN_REGISTRY_LIST: [(&'static str, InstrCtor); 8] = [
    ("label", parse_generic::<Label>),
    ("line", parse_generic::<Line>),
    ("func_info", parse_generic::<FuncInfo>),
    ("allocate", parse_generic::<Allocate>),
    ("init_yregs", parse_generic::<InitYRegs>),
    ("move", parse_generic::<Move>),
    ("x", parse_generic::<XReg>),
    ("y", parse_generic::<YReg>)
];

type InstrCtor = fn(&OtpErlangTerm) -> Result<Box<dyn Token>, &'static str>;

fn parse_generic<T>(term: &OtpErlangTerm) -> Result<Box<dyn Token>, &'static str>
where
    T: Token,
{
    T::parse(term)
}

static TOKEN_REGISTRY: Lazy<HashMap<&'static str, InstrCtor>> =
    Lazy::new(|| HashMap::from(TOKEN_REGISTRY_LIST));

macro_rules! extrtuple {
    ($tuple_name:ident, $term:ident) => {
        let OtpErlangTerm::OtpErlangTuple($tuple_name) = $term else {
            return Err("Instr term must be a tuple");
        };
    };
}

macro_rules! emptyinstr {
    ($type:ident) => {
        struct $type {}
        impl Token for $type {
            fn parse(term: &OtpErlangTerm) -> Result<Box<dyn Token>, &'static str> {
                Ok(Box::new($type {}))
            }

            fn translate(&self) -> Option<String> {
                None
            }
        }
    };
}

pub fn parse_token(term: &OtpErlangTerm) -> Result<Box<dyn Token>, &'static str> {
    extrtuple!(instr_tuple, term);

    let OtpErlangTerm::OtpErlangAtomUTF8(name_atom) = &instr_tuple[0] else {
        return Err("Instr tuple must contain it's name as 1st element");
    };
    let name = atomutf8_to_string(name_atom)?;

    if !TOKEN_REGISTRY.contains_key(name.as_str()) {
        return parse_generic::<Unresolved>(term);
    }

    TOKEN_REGISTRY[name.as_str()](term)
}

struct Unresolved {
    term: OtpErlangTerm,
}

impl Token for Unresolved {
    fn parse(term: &OtpErlangTerm) -> Result<Box<dyn Token>, &'static str> {
        Ok(Box::new(Unresolved { term: term.clone() }))
    }

    fn translate(&self) -> Option<String> {
        let tr = format!("{:?}", self.term);
        Some(tr)
    }
}

struct Label {
    num: i32,
}

impl Token for Label {
    fn parse(term: &OtpErlangTerm) -> Result<Box<dyn Token>, &'static str> {
        extrtuple!(label_tuple, term);
        let OtpErlangTerm::OtpErlangInteger(num) = &label_tuple[1] else {
            return Err("Label must have integer as it's 2nd element");
        };

        let label = Label { num: *num };
        Ok(Box::new(label))
    }

    fn translate(&self) -> Option<String> {
        None
    }
}

emptyinstr!(Line);
emptyinstr!(FuncInfo);
emptyinstr!(Allocate);
emptyinstr!(InitYRegs);

struct XReg {
    num: i32,
}

impl Token for XReg {
    fn parse(term: &OtpErlangTerm) -> Result<Box<dyn Token>, &'static str> {
        extrtuple!(xreg_tuple, term);
        let OtpErlangTerm::OtpErlangInteger(num) = &xreg_tuple[1] else {
            return Err("XReg must have integer as it's 2nd element");
        };

        let xreg = XReg { num: *num };
        Ok(Box::new(xreg))
    }

    fn translate(&self) -> Option<String> {
        Some(format!("X{}", self.num))
    }
}

struct YReg {
    num: i32,
}

impl Token for YReg {
    fn parse(term: &OtpErlangTerm) -> Result<Box<dyn Token>, &'static str> {
        extrtuple!(yreg_tuple, term);
        let OtpErlangTerm::OtpErlangInteger(num) = &yreg_tuple[1] else {
            return Err("YReg must have integer as it's 2nd element");
        };

        let yreg = YReg { num: *num };
        Ok(Box::new(yreg))
    }

    fn translate(&self) -> Option<String> {
        Some(format!("Y{}", self.num))
    }
}

struct Move {
    lvalue: Box<dyn Token>,
    rvalue: Box<dyn Token>,
}

impl Token for Move {
    fn parse(term: &OtpErlangTerm) -> Result<Box<dyn Token>, &'static str> {
        extrtuple!(move_tuple, term);
        if (move_tuple.len() != 3) {
            return Err("Move tuple must have 3 elements");
        }

        let rvalue = parse_token(&move_tuple[1])?;
        let lvalue = parse_token(&move_tuple[2])?;
        let move_token = Move {
            rvalue: rvalue,
            lvalue: lvalue,
        };

        Ok(Box::new(move_token))
    }

    fn translate(&self) -> Option<String> {
        let tr1 = self.lvalue.translate().unwrap_or(String::from(""));
        let tr2 = self.rvalue.translate().unwrap_or(String::from(""));
        Some(format!("{} = {}", tr1, tr2))
    }
}
