
// Parse a string of "key:value" form into a tuple of (key, value).
pub fn parse_value(s: &str) -> Result<(&str, &str), String> {
    let pos = s.find(":")
        .ok_or(format!("Invalid value `{}`", s))?;

    let mut result = s.split_at(pos);
    result.1 = &result.1[1..];

    Ok(result)
}
