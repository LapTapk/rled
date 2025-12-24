use erlang::OtpErlangTerm;

pub fn atomutf8_to_string(term: &OtpErlangTerm) -> Result<String, &'static str> {
    let OtpErlangTerm::OtpErlangAtomUTF8(atom) = term else {
        return Err("Is not an atom");
    };

    Ok(String::from(
        std::str::from_utf8(atom).map_err(|_| "Atom cannot be decoded in string")?,
    ))
}
