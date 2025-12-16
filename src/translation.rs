use crate::instructions::parse_instr;
use crate::util::atomutf8_to_string;
use erlang::OtpErlangTerm;
use std::collections::LinkedList;

pub trait Token {
    fn parse<'a>(term: &OtpErlangTerm, slist: SymbolList<'a>) -> Result<Parsed<'a>, &'static str>
    where
        Self: Sized;
    // fn translate(&self);
}

pub enum Reg {
    X { i: i32 },
    Y { i: i32 },
}

pub enum SymbolValue<'a> {
    FuncRes { token: &'a dyn Token },
    Constant { term: &'a OtpErlangTerm },
    Symbol { id: i32 },
}

pub struct Symbol<'a> {
    reg: Reg,
    value: SymbolValue<'a>,
    parent_node: &'a dyn Token,
}

pub type SymbolList<'a> = LinkedList<Symbol<'a>>;

pub struct Parsed<'a> {
    pub token: Box<dyn Token>,
    pub slist: SymbolList<'a>,
}

pub struct Func {
    name: String,
    arity: i32,
    label: i32,
    instrs: Vec<Box<dyn Token>>,
}

pub struct Module {
    name: String,
    funcs: Vec<Box<dyn Token>>,
}

impl Token for Func {
    fn parse<'a>(
        term: &OtpErlangTerm,
        mut slist: SymbolList<'a>,
    ) -> Result<Parsed<'a>, &'static str> {
        let OtpErlangTerm::OtpErlangTuple(func_tuple) = term else {
            return Err("Func term must be a tuple");
        };
        let OtpErlangTerm::OtpErlangAtomUTF8(name_atom) = &func_tuple[1] else {
            return Err("Func tuple must contain its name as 2nd element");
        };
        let OtpErlangTerm::OtpErlangInteger(arity) = func_tuple[2] else {
            return Err("Func tuple must contain arity as 3rd element");
        };
        let OtpErlangTerm::OtpErlangInteger(label) = func_tuple[3] else {
            return Err("Func tuple must contain it's label as 4th element");
        };
        let name = atomutf8_to_string(name_atom)?;

        let OtpErlangTerm::OtpErlangList(instrs_list) = &func_tuple[4] else {
            return Err("Func tuple must contain instruction list as 5th element");
        };

        let mut instrs: Vec<Box<dyn Token>> = Vec::with_capacity(instrs_list.len());
        for instr_term in instrs_list {
            let parsed_instr = parse_instr(instr_term, slist)?;
            instrs.push(parsed_instr.token);
            slist = parsed_instr.slist;
        }

        let token = Func {
            name: name,
            arity: arity,
            label: label,
            instrs: instrs,
        };
        Ok(Parsed {
            token: Box::new(token),
            slist: SymbolList::new(),
        })
    }
}

impl Token for Module {
    fn parse<'a>(
        term: &OtpErlangTerm,
        mut slist: SymbolList<'a>,
    ) -> Result<Parsed<'a>, &'static str> {
        let OtpErlangTerm::OtpErlangTuple(module) = term else {
            return Err("Module term must be a tuple");
        };

        let OtpErlangTerm::OtpErlangAtomUTF8(module_name_atom) = &module[1] else {
            return Err("Module tuple must contain name atom as 2nd element");
        };
        let name = atomutf8_to_string(module_name_atom)?;

        let OtpErlangTerm::OtpErlangList(funcs_beam) = &module[5] else {
            return Err("Module tuple must contain function list as 6th element");
        };
        let mut funcs: Vec<Box<dyn Token>> = Vec::with_capacity(funcs_beam.len());
        for func_beam in funcs_beam {
            let parsed_func = Func::parse(func_beam, slist)?;
            funcs.push(parsed_func.token);
            slist = parsed_func.slist;
        }

        let module = Module {
            name: name,
            funcs: funcs,
        };
        Ok(Parsed {
            token: Box::new(module),
            slist: slist,
        })
    }
}
