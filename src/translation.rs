use crate::util::atomutf8_to_string;
use downcast_rs::{Downcast, impl_downcast};
use erlang::OtpErlangTerm;
use once_cell::sync::Lazy;
use std::any::type_name;
use std::collections::HashMap;
use std::collections::LinkedList;
use std::fs::OpenOptions;
use std::io::Write;

pub fn translate(term: &OtpErlangTerm) -> String {
    let Ok(module) = <Module as TokenMeta>::parse(term) else {
        return "Failed to translate BEAM file".into();
    };

    module
        .translate()
        .unwrap_or("Failed to translate BEAM file".into())
}

type Parsed = Result<Box<dyn Token>, &'static str>;

trait TokenMeta {
    const LEXEME: &'static str;
    fn parse(term: &OtpErlangTerm) -> Parsed;
}

trait Token: Downcast {
    fn translate(&self) -> Option<String>;
}
impl_downcast!(Token);

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
            const LEXEME: &'static str = $name;
            fn parse(term: &OtpErlangTerm) -> Parsed {
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

macro_rules! _parse_next_token {
     ($term:expr, $atom:expr, $($token:ty),*) => {
         {
            let lexeme = atomutf8_to_string($atom)?;
            let token_res = match lexeme.as_str() {
                $(
                    <$token as TokenMeta>::LEXEME => <$token as TokenMeta>::parse,
                )*
                _ => {
                    let log = format!("0:{}:{}:{}:{:?}:{:?};", file!(), line!(), lexeme, $term, [$(type_name::<$token>(),)*]);
                    writeln!(
                        OpenOptions::new()
                            .create(true)
                            .append(true)
                            .open("rled.tmp/rled.log")
                            .unwrap(),
                        "{}",
                        log
                    );
                    <Unresolved as TokenMeta>::parse
                }
            }($term);

            match token_res {
                Ok(token) => token,
                Err(s) => {
                    let log = format!("1:{}:{}:{}:{:?}:{:?};", file!(), line!(), lexeme, $term, [$(type_name::<$token>(),)*]);
                    writeln!(
                        OpenOptions::new()
                            .create(true)
                            .append(true)
                            .open("rled.tmp/rled.log")
                            .unwrap(),
                        "{}",
                        log
                    );
                    <Unresolved as TokenMeta>::parse($term).unwrap()
                }
            }
        }
     }
}

macro_rules! parse_next_token {
    ($term:expr => $($token:ty)|*) => {
        {
            match $term {
                OtpErlangTerm::OtpErlangTuple(tuple) => _parse_next_token!($term, &tuple[0], $($token),*),
                OtpErlangTerm::OtpErlangAtomUTF8(_) => _parse_next_token!($term, $term, $($token),*),
                _ => return Err("Cannot find any matching token productions"),
            }
        }
    };
}

macro_rules! comptoken {
    {$name:ident, $lexeme:literal, $tr:literal} => {
        struct $name;
        impl TokenMeta for $name {
            const LEXEME: &'static str = $lexeme;
            fn parse(term: &OtpErlangTerm) -> Parsed {
                Ok(Box::new($name {}))
            }
        }

        impl Token for $name {
            fn translate(&self) -> Option<String> {
                Some($tr.into())
            }
        }
    }
}

struct Module {
    name: String,
    funcs: Vec<Box<dyn Token>>,
}

impl TokenMeta for Module {
    const LEXEME: &'static str = "beam_file";
    fn parse(term: &OtpErlangTerm) -> Parsed {
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

struct Func {
    name: String,
    arity: i32,
    label: i32,
    instrs: Vec<Box<dyn Token>>,
}

impl TokenMeta for Func {
    const LEXEME: &'static str = "function";
    fn parse(term: &OtpErlangTerm) -> Parsed {
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
                CallExtOnly |
                TestHeap |
                PutList |
                Test
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
    const LEXEME: &'static str = "";
    fn parse(term: &OtpErlangTerm) -> Parsed {
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
    const LEXEME: &'static str = "label";
    fn parse(term: &OtpErlangTerm) -> Parsed {
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
        Some(format!("\nlabel{}:", self.num))
    }
}

struct XReg {
    num: i32,
}

impl TokenMeta for XReg {
    const LEXEME: &'static str = "x";
    fn parse(term: &OtpErlangTerm) -> Parsed {
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
    const LEXEME: &'static str = "y";
    fn parse(term: &OtpErlangTerm) -> Parsed {
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
    const LEXEME: &'static str = "move";
    fn parse(term: &OtpErlangTerm) -> Parsed {
        extrtuple!(move_tuple, term);
        if (move_tuple.len() != 3) {
            return Err("Move tuple must have 3 elements");
        }

        let rvalue = parse_next_token!(&move_tuple[1] => XReg | YReg | Literal | Integer | Atom);
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
    const LEXEME: &'static str = "call_ext";
    fn parse(term: &OtpErlangTerm) -> Parsed {
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
    const LEXEME: &'static str = "extfunc";
    fn parse(term: &OtpErlangTerm) -> Parsed {
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
        let tr = format!("{}:{}({})", self.module, self.name, args);
        Some(tr)
    }
}

struct FLabel {
    num: i32,
}

impl TokenMeta for FLabel {
    const LEXEME: &'static str = "f";
    fn parse(term: &OtpErlangTerm) -> Parsed {
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
        Some(format!("label{}", self.num))
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
    const LEXEME: &'static str = "gc_bif";
    fn parse(term: &OtpErlangTerm) -> Parsed {
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
    const LEXEME: &'static str = "literal";
    fn parse(term: &OtpErlangTerm) -> Parsed {
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
        Some(format!("{:?}", self.s.clone()))
    }
}

struct CallExtLast {
    arity: i32,
    func: Box<dyn Token>,
}

impl TokenMeta for CallExtLast {
    const LEXEME: &'static str = "call_ext_last";
    fn parse(term: &OtpErlangTerm) -> Parsed {
        extrtuple!(callext_tuple, term);
        if callext_tuple.len() != 4 {
            return Err("CallExtLast tuple must be 4 items long");
        }

        let OtpErlangTerm::OtpErlangInteger(arity) = &callext_tuple[1] else {
            return Err("CallExtLast tuple must contain arity integer as 2nd element");
        };
        let func = parse_next_token!(&callext_tuple[2] => ExtFunc);

        let call_ext = CallExtLast {
            arity: *arity,
            func: func,
        };

        Ok(Box::new(call_ext))
    }
}

impl Token for CallExtLast {
    fn translate(&self) -> Option<String> {
        Some(format!(
            "return {}",
            self.func.translate().unwrap_or("ERROR".into())
        ))
    }
}

struct CallExtOnly {
    arity: i32,
    func: Box<dyn Token>,
}

impl TokenMeta for CallExtOnly {
    const LEXEME: &'static str = "call_ext_only";
    fn parse(term: &OtpErlangTerm) -> Parsed {
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

struct Integer {
    num: i32,
}

impl TokenMeta for Integer {
    const LEXEME: &'static str = "integer";
    fn parse(term: &OtpErlangTerm) -> Parsed {
        extrtuple!(int_tuple, term);
        let OtpErlangTerm::OtpErlangInteger(num) = &int_tuple[1] else {
            return Err("Integer tuple must contain an integer as 2nd element");
        };
        let integer = Integer { num: *num };
        Ok(Box::new(integer))
    }
}

impl Token for Integer {
    fn translate(&self) -> Option<String> {
        Some(format!("{}", self.num))
    }
}

struct PutList {
    head: Box<dyn Token>,
    tail: Box<dyn Token>,
    store: Box<dyn Token>,
    tail_is_nil: bool,
}

impl TokenMeta for PutList {
    const LEXEME: &'static str = "put_list";
    fn parse(term: &OtpErlangTerm) -> Parsed {
        extrtuple!(putlist_tuple, term);

        if putlist_tuple.len() != 4 {
            return Err("Putlist tuple must be 4 items long");
        }

        let head = parse_next_token!(&putlist_tuple[1] => YReg | XReg);

        let tail = parse_next_token!(&putlist_tuple[2] => YReg | XReg | Nil);
        let mut tail_is_nil = false;
        if let Some(_) = tail.downcast_ref::<Nil>() {
            tail_is_nil = true;
        }

        let store = parse_next_token!(&putlist_tuple[3] => YReg | XReg);

        let put_list = PutList {
            head: head,
            tail: tail,
            store: store,
            tail_is_nil,
        };

        Ok(Box::new(put_list))
    }
}

impl Token for PutList {
    fn translate(&self) -> Option<String> {
        let head_tr = self.head.translate().unwrap_or("".into());
        let store_tr = self.store.translate().unwrap_or("".into());
        if (self.tail_is_nil) {
            Some(format!("{} = [{}]", store_tr, head_tr))
        } else {
            let tail_tr = self.tail.translate().unwrap_or("".into());
            Some(format!("{} = [{} | {}]", store_tr, head_tr, tail_tr))
        }
    }
}

struct Nil {}

impl TokenMeta for Nil {
    const LEXEME: &'static str = "nil";
    fn parse(term: &OtpErlangTerm) -> Parsed {
        Ok(Box::new(Nil {}))
    }
}

impl Token for Nil {
    fn translate(&self) -> Option<String> {
        Some("nil".into())
    }
}

struct Atom {
    name: String,
}

impl TokenMeta for Atom {
    const LEXEME: &'static str = "atom";
    fn parse(term: &OtpErlangTerm) -> Parsed {
        extrtuple!(atom_tuple, term);
        let name = atomutf8_to_string(&atom_tuple[1])?;
        let atom = Atom { name: name };
        Ok(Box::new(atom))
    }
}

impl Token for Atom {
    fn translate(&self) -> Option<String> {
        Some(self.name.clone())
    }
}

struct TInteger {
    num: i32,
}

impl TokenMeta for TInteger {
    const LEXEME: &'static str = "t_integer";
    fn parse(term: &OtpErlangTerm) -> Parsed {
        extrtuple!(tint_tuple, term);
        let OtpErlangTerm::OtpErlangInteger(num) = &tint_tuple[1] else {
            return Err("TInteger tuple must contain an integer as 2nd element");
        };
        let integer = TInteger { num: *num };
        Ok(Box::new(integer))
    }
}

impl Token for TInteger {
    fn translate(&self) -> Option<String> {
        Some(format!("{}", self.num))
    }
}

struct Tr {
    reg: Box<dyn Token>,
    ty: Box<dyn Token>,
}

impl TokenMeta for Tr {
    const LEXEME: &'static str = "tr";
    fn parse(term: &OtpErlangTerm) -> Parsed {
        extrtuple!(tr_tuple, term);

        if tr_tuple.len() != 3 {
            return Err("Tr tuple must be 3 items long");
        }

        let reg = parse_next_token!(&tr_tuple[1] => YReg | XReg);
        let ty = parse_next_token!(&tr_tuple[2] => TInteger);

        let tr = Tr { reg, ty };
        Ok(Box::new(tr))
    }
}

impl Token for Tr {
    fn translate(&self) -> Option<String> {
        let reg_tr = self.reg.translate().unwrap_or("".into());
        Some(format!("{}", reg_tr))
    }
}

struct Test {
    comp: Box<dyn Token>,
    fail: Box<dyn Token>,
    args: Vec<Box<dyn Token>>,
}

impl TokenMeta for Test {
    const LEXEME: &'static str = "test";
    fn parse(term: &OtpErlangTerm) -> Parsed {
        extrtuple!(test_tuple, term);
        if test_tuple.len() != 4 {
            return Err("Test tuple must be 4 items long");
        }

        let comp = parse_next_token!(&test_tuple[1] => IsGe | IsEqExact);
        let fail = parse_next_token!(&test_tuple[2] => FLabel);
        let OtpErlangTerm::OtpErlangList(args_terms) = &test_tuple[3] else {
            return Err("Test tuple must have args list as 4th element");
        };

        let mut args: Vec<Box<dyn Token>> = Vec::with_capacity(args_terms.len());
        for arg_term in args_terms {
            let parsed_arg = parse_next_token!(arg_term => Integer | Tr | XReg | YReg);
            args.push(parsed_arg);
        }

        let test = Test { comp, fail, args };
        Ok(Box::new(test))
    }
}

impl Token for Test {
    fn translate(&self) -> Option<String> {
        let mut tr_arg1 = String::from("");
        let mut tr_arg2 = String::from("");
        if self.args.len() >= 2 {
            tr_arg1 = self.args[0].translate().unwrap_or("".into());
            tr_arg2 = self.args[1].translate().unwrap_or("".into());
        }

        let tr_comp = self.comp.translate().unwrap_or("".into());
        let tr_fail = self.fail.translate().unwrap_or("".into());
        Some(format!(
            "\nif {} {} {} fail then goto {}",
            tr_arg1, tr_comp, tr_arg2, tr_fail
        ))
    }
}

comptoken! {IsGe, "is_ge", ">="}
comptoken! {IsEqExact, "is_eq_exact", "=="}

emptyinstr!(Line, "line");
emptyinstr!(FuncInfo, "func_info");
emptyinstr!(Allocate, "allocate");
emptyinstr!(InitYRegs, "init_yregs");
emptyinstr!(TestHeap, "test_heap");
