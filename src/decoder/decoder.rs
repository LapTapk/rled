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

    Module {
        name: module_name,
        funcs: Vec::new()
    }
}
