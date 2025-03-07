/// Find all occurrences of a character outside of quotes in a string.
/// Ej: find_all_outside_quotes(":':':", ':') -> [0, 4]
///
/// Escaped values are ignored
pub fn find_all_outside_quotes<T: AsRef<str>>(str: T, needle: char) -> Vec<usize> {
    let mut indexes = Vec::new();

    let mut quoted = false;
    let mut escaped = false;
    for (i, c) in str.as_ref().chars().enumerate() {
        match (escaped, quoted, c) {
            (_, _, '\\') => escaped = !escaped,
            (false, false, '"') => quoted = true,
            (false, false, _) => {
                if c == needle {
                    indexes.push(i);
                }
            }
            (false, true, '"') => quoted = false,
            (_, _, _) => {}
        }
    }

    indexes
}

#[cfg(test)]
mod test {
    #[test]
    fn test_find_all_outside_quotes() {
        let needle = ':';
        let empty = Vec::<usize>::new();

        // Simple case
        let result = super::find_all_outside_quotes(r#":"::":"#, needle);
        assert_eq!(vec![0, 5], result);

        // No colons
        let result = super::find_all_outside_quotes("abcd", needle);
        assert_eq!(Vec::<usize>::new(), result);

        // All colons inside quotes
        let result = super::find_all_outside_quotes(r#""a:b:c""#, needle);
        assert_eq!(Vec::<usize>::new(), result);

        // Colons inside and outside quotes
        let result = super::find_all_outside_quotes(r#":"a:b":c:d"#, needle);
        assert_eq!(vec![0, 6, 8], result);

        // Escaped quotes
        let result = super::find_all_outside_quotes(r#""a\"b:c\":d:e""#, needle);
        assert_eq!(vec![9, 11], result);

        // Escaped escape
        let result = super::find_all_outside_quotes(r#"\\":""#, needle);
        assert_eq!(empty, result);
    }
}
