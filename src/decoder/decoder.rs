use erlang::OtpErlangTerm;

pub struct Instr {
    name: String,
    args: Vec<OtpErlangTerm>,
}

pub struct Func {
    name: String,
    arity: i32,
    label: i32,
    instrs: Vec<Instr>,
}

pub struct Module {
    name: String,
    funcs: Vec<Func>,
}

fn atomutf8_to_string(data: &Vec<u8>) -> Option<String> {
    Some(String::from(std::str::from_utf8(data).ok()?))
}

fn decode_instr(instr_beam: &OtpErlangTerm) -> Option<Instr> {
    Some(Instr {
        name: String::from(""),
        args: Vec::new(),
    })
}

fn decode_func(func_beam: &OtpErlangTerm) -> Option<Func> {
    let OtpErlangTerm::OtpErlangTuple(func_tuple) = func_beam else {
        return None;
    };
    let OtpErlangTerm::OtpErlangAtomUTF8(name_atom) = &func_tuple[1] else {
        return None;
    };
    let OtpErlangTerm::OtpErlangInteger(arity) = func_tuple[2] else {
        return None;
    };
    let OtpErlangTerm::OtpErlangInteger(label) = func_tuple[3] else {
        return None;
    };
    let name = atomutf8_to_string(name_atom)?;

    let OtpErlangTerm::OtpErlangList(instrs_beam) = &func_tuple[4] else {
        return None;
    };
    let mut instrs: Vec<Instr> = Vec::with_capacity(instrs_beam.len());
    for instr_beam in instrs_beam {
        instrs.push(decode_instr(instr_beam)?);
    }

    Some(Func {
        name: name,
        arity: arity,
        label: label,
        instrs: instrs,
    })
}

pub fn decode(data: Vec<u8>) -> Option<Module> {
    let terms = erlang::binary_to_term(&data).expect("Invalid data format");

    let OtpErlangTerm::OtpErlangTuple(module) = terms else {
        return None;
    };

    let OtpErlangTerm::OtpErlangAtomUTF8(module_name_atom) = &module[1] else {
        return None;
    };
    let module_name = atomutf8_to_string(module_name_atom)?;

    let OtpErlangTerm::OtpErlangList(funcs_beam) = &module[5] else {
        return None;
    };
    let mut funcs: Vec<Func> = Vec::with_capacity(funcs_beam.len());
    for func_beam in funcs_beam {
        funcs.push(decode_func(func_beam)?);
    }

    Some(Module {
        name: module_name,
        funcs: funcs,
    })
}
