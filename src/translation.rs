use crate::instructions::parse_token;
use crate::util::atomutf8_to_string;
use erlang::OtpErlangTerm;
use std::collections::LinkedList;

pub trait Token {
    fn parse(term: &OtpErlangTerm) -> Result<Box<dyn Token>, &'static str>
    where
        Self: Sized;
    fn translate(&self) -> Option<String>;
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
    fn parse<'a>(term: &OtpErlangTerm) -> Result<Box<dyn Token>, &'static str> {
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
            let parsed_instr = parse_token(instr_term)?;
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
            .map(|x| format!("Arg{}", x))
            .collect::<Vec<_>>()
            .join(", ");
        let mut tr = vec![format!("{}({}) ->", self.name, args)];
        for instr in &self.instrs {
            let instr_tr_option = instr.translate();
            if let Some(instr_tr) = instr_tr_option {
                let instr_tr_in_func = String::from("    ") + instr_tr.as_str().replace("\n", "\n    ").as_str();
                tr.push(instr_tr_in_func);
            }
        }
        Some(tr.join("\n"))
    }
}

impl Token for Module {
    fn parse<'a>(term: &OtpErlangTerm) -> Result<Box<dyn Token>, &'static str> {
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
            let parsed_func = Func::parse(func_beam)?;
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
            if let Some(func_tr) = func_tr_option {
                tr.push(func_tr)
            }
        }
        Some(tr.join("\n"))
    }
}

