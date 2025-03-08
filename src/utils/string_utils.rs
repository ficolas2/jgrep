use std::{iter::Peekable, str::Chars};

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

pub fn wildcard_match_internal(mut haystack: Chars, mut needle: Peekable<Chars>) -> bool {
    loop {
        match needle.peek() {
            Some('*') => {
                let mut next_haystack = haystack.clone();
                let c = next_haystack.next();
                if c.is_some() && wildcard_match_internal(next_haystack, needle.clone()) {
                    return true;
                }
                needle.next();
            }
            Some('?') => {
                needle.next();
                haystack.next();
            }
            Some(c) => {
                if Some(*c) == haystack.next() {
                    needle.next();
                } else {
                    return false;
                }
            }
            None => {
                return haystack.next().is_none();
            }
        }
    }
}

pub fn wildcard_match(haystack: &str, needle: &str) -> bool {
    let haystack_chars = haystack.chars();
    let needle_chars = needle.chars().peekable();
    wildcard_match_internal(haystack_chars, needle_chars)
}

#[cfg(test)]
mod test {
    use crate::utils::string_utils::wildcard_match;

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

    #[test]
    fn test_wildcard_match() {
        // No wildcard
        assert!(wildcard_match("", "")); // Empty
        assert!(wildcard_match("abc", "abc")); // Exact
        
        // *
        // Matches
        assert!(wildcard_match("abc", "a*c")); // Wildcard matches one
        assert!(wildcard_match("abc", "a*")); // Wildcard matches to the end
        assert!(wildcard_match("abc", "*c")); // Wildcard matches from the start
        assert!(wildcard_match("abc", "*")); // Wildcard matches everything
        assert!(wildcard_match("abc", "*b*")); // Wildcard matches beginning and end
        assert!(wildcard_match("abc", "a**c")); // Double wildcard center
        assert!(wildcard_match("abc", "**b**")); // Double wildcard outside
        assert!(wildcard_match("abc", "*abc*")); // Wildcard matches nothing
        assert!(wildcard_match("abc", "a****c")); // lots of wildcards

        // No matches
        assert!(!wildcard_match("abc", "a")); // No wildcard
        assert!(!wildcard_match("abc", "b*")); // Wildcard end
        assert!(!wildcard_match("abc", "*b")); // Wildcard start
        assert!(!wildcard_match("abc", "*d*")); // Multiple wildcards

        // ?
        assert!(wildcard_match("abc", "a?c")); // Single wildcard
        assert!(wildcard_match("abc", "??c")); // Double wildcard
        assert!(wildcard_match("abc", "?bc")); // Start
        assert!(wildcard_match("abc", "ab?")); // End
    }
}
