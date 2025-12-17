use crate::util::atomutf8_to_string;
use erlang::OtpErlangTerm;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::collections::LinkedList;

pub trait Token {
    fn parse(term: &OtpErlangTerm) -> Result<Box<dyn Token>, &'static str>
    where
        Self: Sized;
    fn translate(&self) -> Option<String>;
}

macro_rules! prodlist {
    ($( $type:ident ),*) => {
        vec![ $( $type::parse, )* ]
    }
}

macro_rules! parse_prolog {
    ($tuple_name:ident, $term:ident, $name:literal) => {
        let OtpErlangTerm::OtpErlangTuple($tuple_name) = $term else {
            return Err("Token term must be a tuple");
        };
        if atomutf8_to_string(&$tuple_name[0])?.as_str() != $name {
            return Err("Did not match token type");
        }
    };
}

macro_rules! emptyinstr {
    ($type:ident, $name:literal) => {
        struct $type {}
        impl Token for $type {
            fn parse(term: &OtpErlangTerm) -> Result<Box<dyn Token>, &'static str> {
                parse_prolog!(tuple, term, $name);
                Ok(Box::new($type {}))
            }

            fn translate(&self) -> Option<String> {
                None
            }
        }
    };
}

type Production = fn(&OtpErlangTerm) -> Result<Box<dyn Token>, &'static str>;

fn select_production(term: &OtpErlangTerm, prods: &Vec<Production>) -> Box<dyn Token> {
    for (i, prod) in prods.iter().enumerate() {
        match prod(term) {
            Ok(t) => return t,
            _ => {}
        };
    }

    Unresolved::parse(term).unwrap()
}

pub struct Module {
    name: String,
    funcs: Vec<Box<dyn Token>>,
}

impl Token for Module {
    fn parse(term: &OtpErlangTerm) -> Result<Box<dyn Token>, &'static str> {
        parse_prolog!(module, term, "beam_file");

        let name = atomutf8_to_string(&module[1])?;

        let OtpErlangTerm::OtpErlangList(funcs_beam) = &module[5] else {
            return Err("Module tuple must contain function list as 6th element");
        };
        let mut funcs: Vec<Box<dyn Token>> = Vec::with_capacity(funcs_beam.len());
        for func_beam in funcs_beam {
            let parsed_func = select_production(func_beam, &prodlist!(Func));
            funcs.push(parsed_func);
        }

        let module = Module {
            name: name,
            funcs: funcs,
        };
        Ok(Box::new(module))
    }

    fn translate(&self) -> Option<String> {
        let mut tr = vec![format!("-module({}).", self.name)];
        for func in &self.funcs {
            let func_tr_option = func.translate();
            let Some(func_tr) = func_tr_option else {
                continue;
            };
            tr.push(func_tr)
        }
        Some(tr.join("\n"))
    }
}

pub struct Func {
    name: String,
    arity: i32,
    label: i32,
    instrs: Vec<Box<dyn Token>>,
}

impl Token for Func {
    fn parse(term: &OtpErlangTerm) -> Result<Box<dyn Token>, &'static str> {
        parse_prolog!(func_tuple, term, "function");

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
            let parsed_instr = select_production(
                instr_term,
                &prodlist!(
                    Move, Label, FuncInfo, Allocate, InitYRegs, CallExt, ExtFunc, GcBif, Line
                ),
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
        parse_prolog!(label_tuple, term, "label");
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

emptyinstr!(Line, "line");
emptyinstr!(FuncInfo, "func_info");
emptyinstr!(Allocate, "allocate");
emptyinstr!(InitYRegs, "init_yregs");

struct XReg {
    num: i32,
}

impl Token for XReg {
    fn parse(term: &OtpErlangTerm) -> Result<Box<dyn Token>, &'static str> {
        parse_prolog!(xreg_tuple, term, "x");
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
        parse_prolog!(yreg_tuple, term, "y");
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
        parse_prolog!(move_tuple, term, "move");
        if (move_tuple.len() != 3) {
            return Err("Move tuple must have 3 elements");
        }

        let rvalue = select_production(&move_tuple[1], &prodlist!(XReg, YReg, Literal));
        let lvalue = select_production(&move_tuple[2], &prodlist!(XReg, YReg));
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
        parse_prolog!(callext_tuple, term, "call_ext");
        if callext_tuple.len() != 3 {
            return Err("CallExt tuple must be 3 items long");
        }

        let OtpErlangTerm::OtpErlangInteger(arity) = &callext_tuple[1] else {
            return Err("CallExt tuple must contain arity integer as 2nd element");
        };
        let func = select_production(&callext_tuple[2], &prodlist!(ExtFunc));

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
        parse_prolog!(extfunc_tuple, term, "extfunc");
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
        parse_prolog!(flabel_tuple, term, "f");
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
        parse_prolog!(gcbif_tuple, term, "gc_bif");
        if gcbif_tuple.len() != 6 {
            return Err("GcBif tuple must be 6 items long");
        }

        let name = atomutf8_to_string(&gcbif_tuple[1])?;

        let fallback = <FLabel as Token>::parse(&gcbif_tuple[2])?;

        let OtpErlangTerm::OtpErlangInteger(arity) = &gcbif_tuple[3] else {
            return Err("GcBif tuple must contain arity integer as 5th element");
        };

        let OtpErlangTerm::OtpErlangList(args_terms) = &gcbif_tuple[4] else {
            return Err("GcBif tuple must contain args list as 5th element");
        };
        let mut args: Vec<Box<dyn Token>> = Vec::with_capacity(args_terms.len());
        for term in args_terms {
            let token = select_production(term, &prodlist!(XReg, YReg));
            args.push(token);
        }

        let store = select_production(term, &prodlist!(XReg, YReg));

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

struct Literal {
    s: String
}

impl Token for Literal {
    fn parse(term: &OtpErlangTerm) -> Result<Box<dyn Token>, &'static str> {
        parse_prolog!(literal_tuple, term, "literal");
        let OtpErlangTerm::OtpErlangString(s) = &literal_tuple[1] else {
            return Err("Literal tuple must contain a string as the 2nd element");
        };
        
        let literal = Literal { s: String::from(std::str::from_utf8(s).map_err(|_| "Cannot extract string")?) };
        Ok(Box::new(literal))
    }

    fn translate(&self) -> Option<String> {
        Some(format!("\"{}\"", self.s.clone()))
    }
}
