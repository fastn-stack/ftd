pub fn identifier<'input>(
    scanner: &'input mut fastn_p1::parser::Scanner<'input>,
) -> Option<fastn_p1::Identifier<'input>> {
    let first = scanner.peek()?;
    // the first character should be is_alphabetic or `_`
    if !first.is_alphabetic() && first != '_' {
        return None;
    }

    // later characters should be is_alphanumeric or `_` or `-`
    let span = scanner.take_while(|c| c.is_alphabetic() || c == '_' || c == '-')?;

    Some(fastn_p1::Identifier { name: span })
}

#[cfg(test)]
mod test {
    fastn_p1::tt!(super::identifier);

    #[test]
    fn identifier() {
        // identifiers can't start with a space
        t!(" foo", null, " foo");
        t!("foo", "foo");
        t!("foo bar", "foo", " bar");
        t!("_foo bar", "_foo", " bar");
        t!("_foo-bar", "_foo-bar");
        t!("नम", "नम");
        t!("_नम-जन ", "_नम-जन", " ");
        t!("_नाम-जाने", "_नाम-जाने");
        t!("_नाम-जाने ", "_नाम-जाने", " ");
        // emoji is not a valid identifier
        t!("नम😦", "नम", "😦");
        t!("नम 😦", "नम", " 😦");
        t!("😦नम ", null, "😦नम ");
    }
}
