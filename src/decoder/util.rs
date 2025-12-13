pub fn atomutf8_to_string(data: &Vec<u8>) -> Result<String, &'static str> {
    Ok(String::from(
        std::str::from_utf8(data).map_err(|_| "atom cannot be decoded in string")?,
    ))
}
