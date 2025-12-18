use crate::util::atomutf8_to_string;
use erlang::OtpErlangTerm;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::collections::LinkedList;

pub trait TokenMeta {
    const LEXEM: &'static str;
    fn parse(term: &OtpErlangTerm) -> Result<Box<dyn Token>, &'static str>;
}

pub trait Token {
    fn translate(&self) -> Option<String>;
}

macro_rules! extrtuple {
    ($tuple_name:ident, $term:expr) => {
        let OtpErlangTerm::OtpErlangTuple($tuple_name) = $term else {
            return Err("Token term must be a tuple");
        };
    };
}

macro_rules! emptyinstr {
    ($type:ident, $name:literal) => {
        struct $type {}
        impl TokenMeta for $type {
            const LEXEM: &'static str = $name;
            fn parse(term: &OtpErlangTerm) -> Result<Box<dyn Token>, &'static str> {
                extrtuple!(tuple, term);
                Ok(Box::new($type {}))
            }
        }

        impl Token for $type {
            fn translate(&self) -> Option<String> {
                None
            }
        }
    };
}

type Production = fn(&OtpErlangTerm) -> Result<Box<dyn Token>, &'static str>;

macro_rules! parse_next_token {
    ($term:expr => $($token:ty)|*) => {
        {
            let OtpErlangTerm::OtpErlangTuple(tuple) = $term else {
                return Err("Token term must be a tuple");
            };
            let lexem = atomutf8_to_string(&tuple[0])?;
            let token_res = match lexem.as_str() {
                $(
                    <$token as TokenMeta>::LEXEM => <$token as TokenMeta>::parse,
                )*
                _ => <Unresolved as TokenMeta>::parse
            }($term);

            match token_res {
                Ok(token) => token,
                _ => <Unresolved as TokenMeta>::parse($term).unwrap()
            }
        }
    };
}

pub struct Module {
    name: String,
    funcs: Vec<Box<dyn Token>>,
}

impl TokenMeta for Module {
    const LEXEM: &'static str = "beam_file";
    fn parse(term: &OtpErlangTerm) -> Result<Box<dyn Token>, &'static str> {
        extrtuple!(module, term);

        let name = atomutf8_to_string(&module[1])?;

        let OtpErlangTerm::OtpErlangList(funcs_beam) = &module[5] else {
            return Err("Module tuple must contain function list as 6th element");
        };
        let mut funcs: Vec<Box<dyn Token>> = Vec::with_capacity(funcs_beam.len());
        for func_beam in funcs_beam {
            let parsed_func = parse_next_token!(func_beam => Func);
            funcs.push(parsed_func);
        }

        let module = Module {
            name: name,
            funcs: funcs,
        };
        Ok(Box::new(module))
    }
}

impl Token for Module {
    fn translate(&self) -> Option<String> {
        let mut tr = vec![format!("-module({}).\n", self.name)];
        for func in &self.funcs {
            let func_tr_option = func.translate();
            let Some(func_tr) = func_tr_option else {
                continue;
            };
            tr.push(func_tr)
        }
        Some(tr.join("\n\n"))
    }
}

pub struct Func {
    name: String,
    arity: i32,
    label: i32,
    instrs: Vec<Box<dyn Token>>,
}

impl TokenMeta for Func {
    const LEXEM: &'static str = "function";
    fn parse(term: &OtpErlangTerm) -> Result<Box<dyn Token>, &'static str> {
        extrtuple!(func_tuple, term);

        let name = atomutf8_to_string(&func_tuple[1])?;
        let OtpErlangTerm::OtpErlangInteger(arity) = func_tuple[2] else {
            return Err("Func tuple must contain arity as 3rd element");
        };
        let OtpErlangTerm::OtpErlangInteger(label) = func_tuple[3] else {
            return Err("Func tuple must contain it's label as 4th element");
        };

        let OtpErlangTerm::OtpErlangList(instrs_list) = &func_tuple[4] else {
            return Err("Func tuple must contain instruction list as 5th element");
        };

        let mut instrs: Vec<Box<dyn Token>> = Vec::with_capacity(instrs_list.len());
        for instr_term in instrs_list {
            let parsed_instr = parse_next_token!(
                instr_term =>
                Move |
                Label |
                FuncInfo |
                Allocate |
                InitYRegs |
                CallExt |
                GcBif |
                Line |
                CallExtLast |
                CallExtOnly
            );
            instrs.push(parsed_instr);
        }

        let token = Func {
            name: name,
            arity: arity,
            label: label,
            instrs: instrs,
        };
        Ok(Box::new(token))
    }
}

impl Token for Func {
    fn translate(&self) -> Option<String> {
        let args = (0..self.arity)
            .map(|x| format!("X{}", x))
            .collect::<Vec<_>>()
            .join(", ");
        let mut tr = vec![format!("{}({}) ->", self.name, args)];
        for instr in &self.instrs {
            let instr_tr_option = instr.translate();
            let Some(instr_tr) = instr_tr_option else {
                continue;
            };

            let instr_tr_in_func =
                String::from("    ") + instr_tr.as_str().replace("\n", "\n    ").as_str();
            tr.push(instr_tr_in_func);
        }
        Some(tr.join("\n"))
    }
}

struct Unresolved {
    term: OtpErlangTerm,
}

impl TokenMeta for Unresolved {
    const LEXEM: &'static str = "";
    fn parse(term: &OtpErlangTerm) -> Result<Box<dyn Token>, &'static str> {
        Ok(Box::new(Unresolved { term: term.clone() }))
    }
}

impl Token for Unresolved {
    fn translate(&self) -> Option<String> {
        let tr = format!("{:?}", self.term);
        Some(tr)
    }
}

struct Label {
    num: i32,
}

impl TokenMeta for Label {
    const LEXEM: &'static str = "label";
    fn parse(term: &OtpErlangTerm) -> Result<Box<dyn Token>, &'static str> {
        extrtuple!(label_tuple, term);
        let OtpErlangTerm::OtpErlangInteger(num) = &label_tuple[1] else {
            return Err("Label must have integer as it's 2nd element");
        };

        let label = Label { num: *num };
        Ok(Box::new(label))
    }
}

impl Token for Label {
    fn translate(&self) -> Option<String> {
        None
    }
}

emptyinstr!(Line, "line");
emptyinstr!(FuncInfo, "func_info");
emptyinstr!(Allocate, "allocate");
emptyinstr!(InitYRegs, "init_yregs");

struct XReg {
    num: i32,
}

impl TokenMeta for XReg {
    const LEXEM: &'static str = "x";
    fn parse(term: &OtpErlangTerm) -> Result<Box<dyn Token>, &'static str> {
        extrtuple!(xreg_tuple, term);
        let OtpErlangTerm::OtpErlangInteger(num) = &xreg_tuple[1] else {
            return Err("XReg must have integer as it's 2nd element");
        };

        let xreg = XReg { num: *num };
        Ok(Box::new(xreg))
    }
}

impl Token for XReg {
    fn translate(&self) -> Option<String> {
        Some(format!("X{}", self.num))
    }
}

struct YReg {
    num: i32,
}

impl TokenMeta for YReg {
    const LEXEM: &'static str = "y";
    fn parse(term: &OtpErlangTerm) -> Result<Box<dyn Token>, &'static str> {
        extrtuple!(yreg_tuple, term);
        let OtpErlangTerm::OtpErlangInteger(num) = &yreg_tuple[1] else {
            return Err("YReg must have integer as it's 2nd element");
        };

        let yreg = YReg { num: *num };
        Ok(Box::new(yreg))
    }
}

impl Token for YReg {
    fn translate(&self) -> Option<String> {
        Some(format!("Y{}", self.num))
    }
}

struct Move {
    lvalue: Box<dyn Token>,
    rvalue: Box<dyn Token>,
}

impl TokenMeta for Move {
    const LEXEM: &'static str = "move";
    fn parse(term: &OtpErlangTerm) -> Result<Box<dyn Token>, &'static str> {
        extrtuple!(move_tuple, term);
        if (move_tuple.len() != 3) {
            return Err("Move tuple must have 3 elements");
        }

        let rvalue = parse_next_token!(&move_tuple[1] => XReg | YReg | Literal);
        let lvalue = parse_next_token!(&move_tuple[2] => XReg | YReg);
        let move_token = Move {
            rvalue: rvalue,
            lvalue: lvalue,
        };

        Ok(Box::new(move_token))
    }
}

impl Token for Move {
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

impl TokenMeta for CallExt {
    const LEXEM: &'static str = "call_ext";
    fn parse(term: &OtpErlangTerm) -> Result<Box<dyn Token>, &'static str> {
        extrtuple!(callext_tuple, term);
        if callext_tuple.len() != 3 {
            return Err("CallExt tuple must be 3 items long");
        }

        let OtpErlangTerm::OtpErlangInteger(arity) = &callext_tuple[1] else {
            return Err("CallExt tuple must contain arity integer as 2nd element");
        };
        let func = parse_next_token!(&callext_tuple[2] => ExtFunc);

        let call_ext = CallExt {
            arity: *arity,
            func: func,
        };

        Ok(Box::new(call_ext))
    }
}

impl Token for CallExt {
    fn translate(&self) -> Option<String> {
        self.func.translate()
    }
}

struct ExtFunc {
    module: String,
    name: String,
    arity: i32,
}

impl TokenMeta for ExtFunc {
    const LEXEM: &'static str = "extfunc";
    fn parse(term: &OtpErlangTerm) -> Result<Box<dyn Token>, &'static str> {
        extrtuple!(extfunc_tuple, term);
        if extfunc_tuple.len() != 4 {
            return Err("ExtFunc tuple must contain 4 elements");
        }
        let module = atomutf8_to_string(&extfunc_tuple[1])?;
        let name = atomutf8_to_string(&extfunc_tuple[2])?;

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
}

impl Token for ExtFunc {
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

impl TokenMeta for FLabel {
    const LEXEM: &'static str = "f";
    fn parse(term: &OtpErlangTerm) -> Result<Box<dyn Token>, &'static str> {
        extrtuple!(flabel_tuple, term);
        let OtpErlangTerm::OtpErlangInteger(num) = &flabel_tuple[1] else {
            return Err("FLabel must have integer as it's 2nd element");
        };

        let flabel = FLabel { num: *num };
        Ok(Box::new(flabel))
    }
}

impl Token for FLabel {
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

impl TokenMeta for GcBif {
    const LEXEM: &'static str = "gc_bif";
    fn parse(term: &OtpErlangTerm) -> Result<Box<dyn Token>, &'static str> {
        extrtuple!(gcbif_tuple, term);
        if gcbif_tuple.len() != 6 {
            return Err("GcBif tuple must be 6 items long");
        }

        let name = atomutf8_to_string(&gcbif_tuple[1])?;

        let fallback = parse_next_token!(&gcbif_tuple[2] => FLabel);

        let OtpErlangTerm::OtpErlangInteger(arity) = &gcbif_tuple[3] else {
            return Err("GcBif tuple must contain arity integer as 5th element");
        };

        let OtpErlangTerm::OtpErlangList(args_terms) = &gcbif_tuple[4] else {
            return Err("GcBif tuple must contain args list as 5th element");
        };
        let mut args: Vec<Box<dyn Token>> = Vec::with_capacity(args_terms.len());
        for arg_term in args_terms {
            let token = parse_next_token!(arg_term => XReg | YReg);
            args.push(token);
        }

        let store = parse_next_token!(&gcbif_tuple[5] => XReg | YReg);

        let gcbif = GcBif {
            name: name,
            fallback: fallback,
            arity: *arity,
            args: args,
            store: store,
        };

        Ok(Box::new(gcbif))
    }
}

impl Token for GcBif {
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

struct Literal {
    s: String,
}

impl TokenMeta for Literal {
    const LEXEM: &'static str = "literal";
    fn parse(term: &OtpErlangTerm) -> Result<Box<dyn Token>, &'static str> {
        extrtuple!(literal_tuple, term);
        let OtpErlangTerm::OtpErlangString(s) = &literal_tuple[1] else {
            return Err("Literal tuple must contain a string as the 2nd element");
        };

        let literal = Literal {
            s: String::from(std::str::from_utf8(s).map_err(|_| "Cannot extract string")?),
        };
        Ok(Box::new(literal))
    }
}

impl Token for Literal {
    fn translate(&self) -> Option<String> {
        Some(format!("\"{}\"", self.s.clone()))
    }
}

struct CallExtLast {
    arity: i32,
    func: Box<dyn Token>,
}

impl TokenMeta for CallExtLast {
    const LEXEM: &'static str = "call_ext_last";
    fn parse(term: &OtpErlangTerm) -> Result<Box<dyn Token>, &'static str> {
        extrtuple!(callext_tuple, term);
        if callext_tuple.len() != 4 {
            return Err("CallExtLast tuple must be 4 items long");
        }

        let OtpErlangTerm::OtpErlangInteger(arity) = &callext_tuple[1] else {
            return Err("CallExtLast tuple must contain arity integer as 2nd element");
        };
        let func = parse_next_token!(&callext_tuple[2] => ExtFunc);

        let call_ext = CallExt {
            arity: *arity,
            func: func,
        };

        Ok(Box::new(call_ext))
    }
}

impl Token for CallExtLast {
    fn translate(&self) -> Option<String> {
        self.func.translate()
    }
}

struct CallExtOnly {
    arity: i32,
    func: Box<dyn Token>,
}

impl TokenMeta for CallExtOnly {
    const LEXEM: &'static str = "call_ext_only";
    fn parse(term: &OtpErlangTerm) -> Result<Box<dyn Token>, &'static str> {
        extrtuple!(callext_tuple, term);
        if callext_tuple.len() != 3 {
            return Err("CallExtOnly tuple must be 3 items long");
        }

        let OtpErlangTerm::OtpErlangInteger(arity) = &callext_tuple[1] else {
            return Err("CallExtOnly tuple must contain arity integer as 2nd element");
        };
        let func = parse_next_token!(&callext_tuple[2] => ExtFunc);

        let call_ext = CallExt {
            arity: *arity,
            func: func,
        };

        Ok(Box::new(call_ext))
    }
}

impl Token for CallExtOnly {
    fn translate(&self) -> Option<String> {
        self.func.translate()
    }
}
