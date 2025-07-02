use std::process::exit;

use pattern_node::{IndexPattern, PatternNode};

use crate::{errors::parsing_error::ParsingError, utils::string_utils};

pub mod pattern_node;
pub mod tokenizer;

#[derive(Debug, PartialEq, Clone)]
pub struct Pattern {
    pub path: Vec<PatternNode>,
    pub value: Option<String>,
    pub or: bool,
    pub start_at_root: bool,
}

impl Pattern {
    fn extract_quoted(path_node_str: &str) -> PatternNode {
        let end_char = path_node_str.chars().last().unwrap();
        if end_char != '"' {
            eprintln!("Invalid pattern: Unmatched quotes");
            exit(1);
        }
        PatternNode::Key(path_node_str[1..path_node_str.len() - 1].to_string())
    }

    fn extract_brackets(path_node_str: &str) -> PatternNode {
        let end_char = path_node_str.chars().last().unwrap();
        if end_char != ']' {
            eprintln!("Invalid pattern: Unmatched brackets");
            exit(1);
        }

        if path_node_str.chars().nth(1) == Some('"') {
            Self::extract_quoted(&path_node_str[1..path_node_str.len() - 1])
        } else {
            let inner_str = &path_node_str[1..path_node_str.len() - 1];
            // All [*] or []
            if inner_str.is_empty() || inner_str == "*" {
                return PatternNode::Index(IndexPattern::All);
            }

            // Range + Last N
            if inner_str.contains(':') {
                let parts: Vec<&str> = inner_str.split(':').collect();
                if parts.len() > 2 {
                    eprintln!("Invalid pattern: More than one colon found in brackets");
                    exit(1);
                }

                let start = match parts[0].parse::<i64>() {
                    Ok(start) => start,
                    Err(_) => {
                        eprintln!("Invalid pattern: Could not parse start index in brackets");
                        exit(1);
                    }
                };
                let end = if parts.len() == 2 {
                    parts[1].parse::<usize>().ok()
                } else {
                    None
                };
                // Last N
                if start < 0 {
                    if end.is_some() {
                        eprintln!("Invalid pattern: Negative start index with end index");
                        exit(1);
                    } else {
                        return PatternNode::Index(IndexPattern::LastN(-start as usize));
                    }
                }

                return PatternNode::Index(IndexPattern::Range(start as usize, end));
            }

            // List
            if inner_str.contains(",") {
                let indices: Result<Vec<usize>, _> = inner_str
                    .split(',')
                    .map(|s| s.trim().parse::<usize>())
                    .collect();

                match indices {
                    Ok(indices) => return PatternNode::Index(IndexPattern::List(indices)),
                    Err(_) => {
                        eprintln!("Invalid pattern: Could not parse indices in brackets");
                        exit(1);
                    }
                }
            }

            // Number
            let index = inner_str.parse::<usize>();
            match index {
                Ok(index) => PatternNode::Index(IndexPattern::List(vec![index])),
                Err(_) => PatternNode::Key(inner_str.to_string()),
            }
        }
    }

    fn parse_path(key_str: &str) -> Result<Vec<PatternNode>, ParsingError> {
        // We need to extract the Path nodes All these possible path nodes:
        // .node."quoted".[].[1].["quoted_bracket"]
        let mut trimmed = key_str.trim();

        if trimmed.is_empty() {
            return Ok(vec![]);
        }

        // To avoid matching a $ key when asserting the start
        if trimmed.starts_with("$.") {
            trimmed = &trimmed[1..];
        }

        // Needed for the '.[]' case, to avoid empty path nodes
        if trimmed.starts_with(".") {
            trimmed = &trimmed[1..];
        }

        let dots = string_utils::find_all_outside_quotes(trimmed, '.');
        let brackets = string_utils::find_all_outside_quotes(trimmed, '[');

        // The cuts between the paths
        let mut cuts = [dots, brackets, vec![trimmed.len()]].concat();
        cuts.sort();
        if !cuts.contains(&0) {
            cuts.insert(0, 0);
        }

        cuts.iter()
            .zip(cuts.iter().skip(1))
            .map(|(&start, &end)| {
                let mut start = start;
                if trimmed.chars().nth(start) == Some('.') {
                    start += 1;
                }

                if start == end {
                    return Ok(PatternNode::Recursive());
                }

                let path_node_str = &trimmed[start..end];

                Ok(match trimmed.chars().nth(start).unwrap() {
                    '[' => Self::extract_brackets(path_node_str),
                    '"' => Self::extract_quoted(path_node_str),
                    _ => {
                        if path_node_str.contains("]") {
                            eprintln!("Invalid pattern: Unexpected ]");
                            exit(1);
                        }
                        PatternNode::Key(path_node_str.to_string())
                    }
                })
            })
            .collect()
    }

    fn parse_value(value_str: &str) -> Result<Option<String>, ParsingError> {
        let trimmed = value_str.trim();

        if trimmed.starts_with('.') || trimmed.starts_with("$.") {
            return Ok(None);
        }

        if trimmed.is_empty() {
            Ok(None)
        } else {
            Ok(Some(trimmed.to_string()))
        }
    }

    /// The possibilities for a pattern are:
    /// - If it contains a :, whats on the left are path nodes, whats on the right are values
    /// - If it doesn't, and it starts with a dot (.), then it is a key
    /// - If neither of those is true, then it matches both, path and values.
    pub fn parse(pattern_str: &str) -> Result<Pattern, ParsingError> {
        let first = pattern_str.chars().next();
        let last = pattern_str.chars().last();

        let start_at_root = pattern_str.starts_with("$.");

        #[rustfmt::skip]
        let (start_wildcard, end_wildcard) = if let (Some(first), Some(last)) = (first, last) {
            (
                if matches!(first, '.' | ':' | '*' | '[' | '"') | start_at_root { "" } else { "*" },
                if matches!(last, '.' | ':' | '*' | ']' | '"') { "" } else { "*" },
            )
        } else {
            ("", "")
        };
        let pattern_str = format!("{}{}{}", start_wildcard, pattern_str, end_wildcard,);

        let colons = string_utils::find_all_outside_quotes_and_brackets(&pattern_str, ':');

        let (path, value, or) = match colons.as_slice() {
            [] => {
                let value = Self::parse_value(&pattern_str)?;
                let or = value.is_some();
                (Self::parse_path(&pattern_str)?, value, or)
            }
            [i] => (
                Self::parse_path(&pattern_str[..*i])?,
                Self::parse_value(&pattern_str[i + 1..])?,
                false,
            ),
            [..] => {
                return Err(ParsingError::new(
                    "Invalid pattern: More than one colon found".to_string(),
                ))
            }
        };

        Ok(Pattern {
            path,
            value,
            or,
            start_at_root,
        })
    }
}

#[cfg(test)]
mod test {
    use crate::pattern::{pattern_node::IndexPattern, PatternNode};

    use super::Pattern;

    #[test]
    fn test_path() {
        let pattern = Pattern::parse(".a.b.c").unwrap();

        assert_eq!(
            Pattern {
                path: vec![
                    PatternNode::Key("a".to_string()),
                    PatternNode::Key("b".to_string()),
                    PatternNode::Key("c*".to_string()),
                ],
                value: None,
                or: false,
                start_at_root: false,
            },
            pattern
        );
    }

    #[test]
    fn test_key() {
        let pattern = Pattern::parse(": true").unwrap();

        assert_eq!(
            Pattern {
                path: vec![],
                value: Some("true*".to_string()),
                or: false,
                start_at_root: false,
            },
            pattern
        );
    }

    #[test]
    fn test_path_and_key() {
        let pattern = Pattern::parse(".a.b.c: true").unwrap();

        assert_eq!(
            Pattern {
                path: vec![
                    PatternNode::Key("a".to_string()),
                    PatternNode::Key("b".to_string()),
                    PatternNode::Key("c".to_string()),
                ],
                value: Some("true*".to_string()),
                or: false,
                start_at_root: false,
            },
            pattern
        );
    }

    #[test]
    fn test_quotes() {
        let pattern = Pattern::parse(".\"a\"").unwrap();

        assert_eq!(
            Pattern {
                path: vec![PatternNode::Key("a".to_string()),],
                value: None,
                or: false,
                start_at_root: false,
            },
            pattern
        );
    }

    #[test]
    fn test_brackets() {
        let pattern = Pattern::parse(r#".[][1]["potato"]"#).unwrap();

        assert_eq!(
            Pattern {
                path: vec![
                    PatternNode::Index(IndexPattern::All),
                    PatternNode::Index(IndexPattern::List(vec![1])),
                    PatternNode::Key("potato".to_string())
                ],
                value: None,
                or: false,
                start_at_root: false,
            },
            pattern
        );
    }

    #[test]
    fn test_start_at_root() {
        let pattern = Pattern::parse("$.a.b.c").unwrap();

        assert_eq!(
            Pattern {
                path: vec![
                    PatternNode::Key("a".to_string()),
                    PatternNode::Key("b".to_string()),
                    PatternNode::Key("c*".to_string()),
                ],
                value: None,
                or: false,
                start_at_root: true
            },
            pattern
        );
    }

    #[test]
    fn test_recursive_at_root() {
        let pattern = Pattern::parse("$..[]").unwrap();

        assert_eq!(
            Pattern {
                path: vec![
                    PatternNode::Recursive(),
                    PatternNode::Index(IndexPattern::All),
                ],
                value: None,
                or: false,
                start_at_root: true
            },
            pattern
        )
    }

    #[test]
    fn test_recursive() {
        let pattern = Pattern::parse(".a..b").unwrap();

        assert_eq!(
            Pattern {
                path: vec![
                    PatternNode::Key("a".to_string()),
                    PatternNode::Recursive(),
                    PatternNode::Key("b*".to_string()),
                ],
                value: None,
                or: false,
                start_at_root: false
            },
            pattern
        )
    }

    #[test]
    fn test_index_list() {
        let pattern = Pattern::parse(".a[1,2,3]").unwrap();

        assert_eq!(
            Pattern {
                path: vec![
                    PatternNode::Key("a".to_string()),
                    PatternNode::Index(IndexPattern::List(vec![1, 2, 3])),
                ],
                value: None,
                or: false,
                start_at_root: false,
            },
            pattern
        );
    }

    #[test]
    fn test_index_range() {
        let pattern = Pattern::parse(".a[1:3]").unwrap();

        assert_eq!(
            Pattern {
                path: vec![
                    PatternNode::Key("a".to_string()),
                    PatternNode::Index(IndexPattern::Range(1, Some(3))),
                ],
                value: None,
                or: false,
                start_at_root: false,
            },
            pattern
        )
    }

    #[test]
    fn test_index_range_last() {
        let pattern = Pattern::parse(".[3:]").unwrap();

        assert_eq!(
            Pattern {
                path: vec![
                    PatternNode::Index(IndexPattern::Range(3, None)),
                ],
                value: None,
                or: false,
                start_at_root: false,
            },
            pattern
        )
    }


    #[test]
    fn test_index_last_n() {
        let pattern = Pattern::parse("[-2:]").unwrap();

        assert_eq!(
            Pattern {
                path: vec![
                    PatternNode::Index(IndexPattern::LastN(2)),
                ],
                value: Some("[-2:]".to_string()),
                or: true,
                start_at_root: false,
            },
            pattern
        )
    }
}
