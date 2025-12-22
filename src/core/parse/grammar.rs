use erlang::OtpErlangTerm;

pub fn expect_tuple<'a>(term: &'a OtpErlangTerm, l: usize) -> Result<&'a Vec<OtpErlangTerm>, String> {
    let OtpErlangTerm::OtpErlangTuple(tuple) = term else {
        return Err("This token must be a tuple".into());
    };

    if tuple.len() != l {
        return Err(format!("This token's tuple must be {} items long", l))
    }

    Ok(tuple)
}

pub fn expect_integer(term: &OtpErlangTerm) -> Result<i32, String> {
    let OtpErlangTerm::OtpErlangInteger(num) = term else {
        return Err("This token must be an integer".into());
    };

    Ok(*num)
}

pub fn expect_list<'a>(term: &'a OtpErlangTerm) -> Result<&'a Vec<OtpErlangTerm>, String> {
    let OtpErlangTerm::OtpErlangList(list) = term else {
        return Err("This token must be a list".into());
    };

    Ok(list)
}

pub fn expect_string(term: &OtpErlangTerm) -> Result<String, String> {
    let OtpErlangTerm::OtpErlangString(s) = term else {
        return Err("This token must be a string".into());
    };

    Ok(String::from(std::str::from_utf8(s).map_err(|_| "Cannot extract string")?))
}
