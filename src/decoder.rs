use erlang::OtpErlangTerm;

#[derive(Debug)]
pub struct Instr {
    name: String,
    args: Vec<OtpErlangTerm>,
}

#[derive(Debug)]
pub struct Func {
    name: String,
    arity: i32,
    label: i32,
    instrs: Vec<Instr>,
}

#[derive(Debug)]
pub struct Module {
    name: String,
    funcs: Vec<Func>,
}

fn atomutf8_to_string(data: &Vec<u8>) -> Result<String, &'static str> {
    Ok(String::from(std::str::from_utf8(data).map_err(|_| {"atom cannot be decoded in string"})?))
}

fn decode_instr(instr_term: &OtpErlangTerm) -> Result<Instr, &'static str> {
    let OtpErlangTerm::OtpErlangTuple(instr_tuple) = instr_term else {
        return Err("Instr term must be a tuple");
    };
    let OtpErlangTerm::OtpErlangAtomUTF8(name_atom) = &instr_tuple[0] else {
        return Err("Instr tuple must contain it's name as 1st element");
    };
    let name = atomutf8_to_string(name_atom)?;

    Ok(Instr {
        name: name,
        args: instr_tuple.clone()
    })
}

fn decode_func(func_beam: &OtpErlangTerm) -> Result<Func, &'static str> {
    let OtpErlangTerm::OtpErlangTuple(func_tuple) = func_beam else {
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
    let mut instrs: Vec<Instr> = Vec::with_capacity(instrs_list.len());
    for instr_term in instrs_list {
        instrs.push(decode_instr(instr_term)?);
    }

    Ok(Func {
        name: name,
        arity: arity,
        label: label,
        instrs: instrs,
    })
}

pub fn decode(data: Vec<u8>) -> Result<Module, &'static str> {
    let terms = erlang::binary_to_term(&data).expect("Invalid data format");

    let OtpErlangTerm::OtpErlangTuple(module) = terms else {
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
        funcs.push(decode_func(func_beam)?);
    }

    Ok(Module {
        name: module_name,
        funcs: funcs,
    })
}
