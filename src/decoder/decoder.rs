use erlang::OtpErlangTerm;

pub struct Instr {
   name: String,
   args: Vec<OtpErlangTerm>
}

pub struct Func {
    name: String,
    arity: i8,
    label: i64,
    instrs: Vec<Instr>
}

pub struct Module {
    name: String,
    funcs: Vec<Func>   
}

fn decode_func(func_beam: &OtpErlangTerm) -> Func {
    Func {
        name: String::from(""),
        arity: 0,
        label: 0,
        instrs: Vec::new()
    }
}

pub fn decode(data: Vec<u8>) -> Module {
    let terms = erlang::binary_to_term(&data).expect("Invalid data format");

    // Extracting module name
    let OtpErlangTerm::OtpErlangTuple(module) = terms else {
        panic!("Data to decode must be a module tuple!")
    };
    let OtpErlangTerm::OtpErlangAtomUTF8(module_name_atom) = &module[0] else {
        panic!("Module tuple must contain module's name as first element")
    };
    let module_name = String::from(std::str::from_utf8(module_name_atom).expect("Module name is not in UTF8"));

    // Decoding functions
    let OtpErlangTerm::OtpErlangTuple(funcs_beam) = &module[5] else {
        panic!("Module tuple must contain vector of functions as 6th element")
    };
    let mut funcs: Vec<Func> = Vec::with_capacity(funcs_beam.len());
    for func_beam in funcs_beam {
        funcs.push(decode_func(func_beam));
    }
    
    Module {
        name: module_name,
        funcs: funcs
    }
}
