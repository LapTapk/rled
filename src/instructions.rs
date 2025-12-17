use crate::translation::*;
use crate::util::atomutf8_to_string;
use erlang::OtpErlangTerm;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::collections::LinkedList;

static TOKEN_REGISTRY_LIST: [(&'static str, InstrCtor); 12] = [
    ("label", parse_generic::<Label>),
    ("line", parse_generic::<Line>),
    ("func_info", parse_generic::<FuncInfo>),
    ("allocate", parse_generic::<Allocate>),
    ("init_yregs", parse_generic::<InitYRegs>),
    ("move", parse_generic::<Move>),
    ("x", parse_generic::<XReg>),
    ("y", parse_generic::<YReg>),
    ("call_ext", parse_generic::<CallExt>),
    ("extfunc", parse_generic::<ExtFunc>),
    ("f", parse_generic::<FLabel>),
    ("gc_bif", parse_generic::<GcBif>)
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

struct CallExt {
    arity: i32,
    func: Box<dyn Token>,
}

impl Token for CallExt {
    fn parse(term: &OtpErlangTerm) -> Result<Box<dyn Token>, &'static str> {
        extrtuple!(callext_tuple, term);
        if callext_tuple.len() != 3 {
            return Err("CallExt tuple must be 3 items long");
        }

        let OtpErlangTerm::OtpErlangInteger(arity) = &callext_tuple[1] else {
            return Err("CallExt tuple must contain arity integer as 2nd element");
        };
        let func = parse_generic::<ExtFunc>(&callext_tuple[2])?;

        let call_ext = CallExt {
            arity: *arity,
            func: func,
        };

        Ok(Box::new(call_ext))
    }

    fn translate(&self) -> Option<String> {
        self.func.translate()
    }
}

struct ExtFunc {
    module: String,
    name: String,
    arity: i32,
}

impl Token for ExtFunc {
    fn parse(term: &OtpErlangTerm) -> Result<Box<dyn Token>, &'static str> {
        extrtuple!(extfunc_tuple, term);
        if extfunc_tuple.len() != 4 {
            return Err("ExtFunc tuple must contain 4 elements");
        }
        let OtpErlangTerm::OtpErlangAtomUTF8(module_atom) = &extfunc_tuple[1] else {
            return Err("ExtFunc tuple must contain module name atom as 2nd element");
        };
        let module = atomutf8_to_string(module_atom)?;

        let OtpErlangTerm::OtpErlangAtomUTF8(name_atom) = &extfunc_tuple[2] else {
            return Err("ExtFunc tuple must contain func name atom as 3rd element");
        };
        let name = atomutf8_to_string(name_atom)?;

        let OtpErlangTerm::OtpErlangInteger(arity) = &extfunc_tuple[3] else {
            return Err("ExtFunc tuple must contain arity integer as 4th element");
        };

        let ext_func = ExtFunc {
            module: module,
            name: name,
            arity: *arity,
        };

        Ok(Box::new(ext_func))
    }

    fn translate(&self) -> Option<String> {
        let args = (0..self.arity)
            .map(|x| format!("X{}", x))
            .collect::<Vec<_>>()
            .join(", ");
        let tr = format!("X0 = {}:{}({})", self.module, self.name, args);
        Some(tr)
    }
}

struct FLabel {
    num: i32,
}

impl Token for FLabel {
    fn parse(term: &OtpErlangTerm) -> Result<Box<dyn Token>, &'static str> {
        extrtuple!(flabel_tuple, term);
        let OtpErlangTerm::OtpErlangInteger(num) = &flabel_tuple[1] else {
            return Err("FLabel must have integer as it's 2nd element");
        };

        let flabel = FLabel { num: *num };
        Ok(Box::new(flabel))
    }

    fn translate(&self) -> Option<String> {
        None
    }
}

struct GcBif {
    name: String,
    fallback: Box<dyn Token>,
    arity: i32,
    args: Vec<Box<dyn Token>>,
    store: Box<dyn Token>,
}

impl Token for GcBif {
    fn parse(term: &OtpErlangTerm) -> Result<Box<dyn Token>, &'static str> {
        extrtuple!(gcbif_tuple, term);
        if gcbif_tuple.len() != 6 {
            return Err("GcBif tuple must be 6 items long");
        }

        let OtpErlangTerm::OtpErlangAtomUTF8(name_atom) = &gcbif_tuple[1] else {
            return Err("GcBif tuple must contain function name atom as 2nd element");
        };
        let name = atomutf8_to_string(name_atom)?;

        let fallback = <FLabel as Token>::parse(&gcbif_tuple[2])?;

        let OtpErlangTerm::OtpErlangInteger(arity) = &gcbif_tuple[3] else {
            return Err("GcBif tuple must contain arity integer as 5th element");
        };

        let OtpErlangTerm::OtpErlangList(args_terms) = &gcbif_tuple[4] else {
            return Err("GcBif tuple must contain args list as 5th element");
        };
        let mut args: Vec<Box<dyn Token>> = Vec::with_capacity(args_terms.len());
        for term in args_terms {
            let token = parse_token(term)?;
            args.push(token);
        }

        let store = parse_token(&gcbif_tuple[5])?;

        let gcbif = GcBif {
            name: name,
            fallback: fallback,
            arity: *arity,
            args: args,
            store: store,
        };

        Ok(Box::new(gcbif))
    }

    fn translate(&self) -> Option<String> {
        let args = self
            .args
            .iter()
            .map(|x| x.translate().unwrap_or(String::from("")))
            .collect::<Vec<_>>()
            .join(", ");
        let store_tr = self.store.translate().unwrap_or(String::from(""));
        let tr = format!("{} = {}({})", store_tr, self.name, args);
        Some(tr)
    }
}
