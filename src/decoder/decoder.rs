use crate::translation::{Token, Instr};
use crate::decoder::instructions::construct_instr;
use crate::decoder::util::atomutf8_to_string;
use erlang::OtpErlangTerm;

pub struct Func {
    name: String,
    arity: i32,
    label: i32,
    instrs: Vec<Box<dyn Instr>>,
}

pub struct Module {
    name: String,
    funcs: Vec<Func>,
}

impl Token for Func {
    fn decode(term: &OtpErlangTerm) -> Result<Self, &'static str> {
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
        let mut instrs: Vec<Box<dyn Instr>> = Vec::with_capacity(instrs_list.len());
        for instr_term in instrs_list {
            instrs.push(construct_instr(instr_term)?);
        }

        Ok(Func {
            name: name,
            arity: arity,
            label: label,
            instrs: instrs,
        })
    }
}

impl Token for Module {
    fn decode(term: &OtpErlangTerm) -> Result<Module, &'static str> {
        let OtpErlangTerm::OtpErlangTuple(module) = term else {
            return Err("Module term must be a tuple");
        };

        let OtpErlangTerm::OtpErlangAtomUTF8(module_name_atom) = &module[1] else {
            return Err("Module tuple must contain name atom as 2nd element");
        };
        let module_name = atomutf8_to_string(module_name_atom)?;

        let OtpErlangTerm::OtpErlangList(funcs_beam) = &module[5] else {
            return Err("Module tuple must contain function list as 6th element");
        };
        let mut funcs: Vec<Func> = Vec::with_capacity(funcs_beam.len());
        for func_beam in funcs_beam {
            funcs.push(Func::decode(func_beam)?);
        }

        Ok(Module {
            name: module_name,
            funcs: funcs,
        })
    }
}
