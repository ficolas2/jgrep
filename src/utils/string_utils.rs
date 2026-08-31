use std::{iter::Peekable, str::Chars};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaseSensitivity {
    Smart,
    Sensitive,
    Insensitive,
}

impl CaseSensitivity {
    pub fn should_ignore_case(self, pattern: &str) -> bool {
        match self {
            Self::Insensitive => true,
            Self::Sensitive => false,
            Self::Smart => !pattern.chars().any(char::is_uppercase),
        }
    }
}

fn chars_match(left: char, right: char, ignore_case: bool) -> bool {
    if ignore_case {
        left.to_lowercase().eq(right.to_lowercase())
    } else {
        left == right
    }
}

fn wildcard_match_internal(
    mut haystack: Chars,
    mut needle: Peekable<Chars>,
    ignore_case: bool,
) -> bool {
    loop {
        match needle.peek() {
            Some('*') => {
                let mut next_haystack = haystack.clone();
                let c = next_haystack.next();
                if c.is_some() && wildcard_match_internal(next_haystack, needle.clone(), ignore_case) {
                    return true;
                }
                needle.next();
            }
            Some('?') => {
                needle.next();
                haystack.next();
            }
            // A backslash makes the next character literal, so that `*`, `?` and `\` itself can be
            // matched. A trailing one has nothing to escape, and stands for itself.
            Some('\\') => {
                needle.next();
                let escaped = needle.next().unwrap_or('\\');
                if Some(escaped) != haystack.next() {
                    return false;
                }
            }
            Some(c) => {
                let next = haystack.next();
                if next.map(|value| chars_match(value, *c, ignore_case)).unwrap_or(false) {
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
    wildcard_match_with_case(haystack, needle, CaseSensitivity::Sensitive)
}

pub fn wildcard_match_with_case(
    haystack: &str,
    needle: &str,
    case_sensitivity: CaseSensitivity,
) -> bool {
    let ignore_case = case_sensitivity.should_ignore_case(needle);
    let haystack_chars = haystack.chars();
    let needle_chars = needle.chars().peekable();
    wildcard_match_internal(haystack_chars, needle_chars, ignore_case)
}

#[cfg(test)]
mod test {
    use crate::utils::string_utils::wildcard_match;

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

        // \
        assert!(wildcard_match("a*c", r"a\*c")); // Literal *
        assert!(!wildcard_match("abc", r"a\*c")); // Literal * does not match another char
        assert!(wildcard_match("a?c", r"a\?c")); // Literal ?
        assert!(!wildcard_match("abc", r"a\?c")); // Literal ? does not match another char
        assert!(wildcard_match(r"a\c", r"a\\c")); // Literal backslash
        assert!(wildcard_match("a\"c", r#"a\"c"#)); // Literal quote
        assert!(wildcard_match(r"a\", r"a\")); // Trailing backslash stands for itself
        assert!(wildcard_match("a*c", r"*\**")); // Escaped and unescaped in one needle

        // ?
        assert!(wildcard_match("abc", "a?c")); // Single wildcard
        assert!(wildcard_match("abc", "??c")); // Double wildcard
        assert!(wildcard_match("abc", "?bc")); // Start
        assert!(wildcard_match("abc", "ab?")); // End
    }
}
