use std::process::exit;

use crate::{errors::parsing_error::ParsingError, path_node::PathNode, utils::string_utils};

#[derive(Debug, PartialEq, Clone)]
pub struct Pattern {
    pub path: Vec<PathNode>,
    pub value: Option<String>,
    pub or: bool,
}

impl Pattern {
    fn extract_quoted(path_node_str: &str) -> PathNode {
        let end_char = path_node_str.chars().last().unwrap();
        if end_char != '"' {
            eprintln!("Invalid pattern: Unmatched quotes");
            exit(1);
        }
        PathNode::Key(path_node_str[1..path_node_str.len() - 1].to_string())
    }

    fn extract_brackets(path_node_str: &str) -> PathNode {
        let end_char = path_node_str.chars().last().unwrap();
        if end_char != ']' {
            eprintln!("Invalid pattern: Unmatched brackets");
            exit(1);
        }

        if path_node_str.chars().nth(1) == Some('"') {
            Self::extract_quoted(&path_node_str[1..path_node_str.len() - 1])
        } else {
            if path_node_str.len() == 2 {
                return PathNode::Index(None);
            }
            let index = path_node_str[1..path_node_str.len() - 1].parse::<usize>();
            match index {
                Ok(index) => PathNode::Index(Some(index)),
                Err(_) => PathNode::Key(path_node_str[1..path_node_str.len() - 1].to_string()),
            }
        }
    }

    fn parse_path(key_str: &str) -> Result<Vec<PathNode>, ParsingError> {
        // We need to extract the Path nodes All these possible path nodes:
        // .node."quoted".[].[1].["quoted_bracket"]
        let mut trimmed = key_str.trim();

        if trimmed.is_empty() {
            return Ok(vec![]);
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
                    return Err(ParsingError::new(
                        "Invalid pattern: Empty path node".to_string(),
                    ));
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
                        PathNode::Key(path_node_str.to_string())
                    }
                })
            })
            .collect()
    }

    fn parse_value(value_str: &str) -> Result<Option<String>, ParsingError> {
        let trimmed = value_str.trim();

        if trimmed.starts_with('.') {
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
        let colons = string_utils::find_all_outside_quotes(pattern_str, ':');

        let (path, value, or) = match colons.as_slice() {
            [] => {
                let value = Self::parse_value(pattern_str)?;
                let or = value.is_some();
                (Self::parse_path(pattern_str)?, value, or)
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

        Ok(Pattern { path, value, or })
    }
}

#[cfg(test)]
mod test {
    use crate::pattern::PathNode;

    use super::Pattern;

    #[test]
    fn test_path() {
        let pattern = Pattern::parse(".a.b.c").unwrap();

        assert_eq!(
            Pattern {
                path: vec![
                    PathNode::Key("a".to_string()),
                    PathNode::Key("b".to_string()),
                    PathNode::Key("c".to_string()),
                ],
                value: None,
                or: false,
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
                value: Some("true".to_string()),
                or: false,
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
                    PathNode::Key("a".to_string()),
                    PathNode::Key("b".to_string()),
                    PathNode::Key("c".to_string()),
                ],
                value: Some("true".to_string()),
                or: false,
            },
            pattern
        );
    }

    #[test]
    fn test_quotes() {
        let pattern = Pattern::parse(".\"a\"").unwrap();

        assert_eq!(
            Pattern {
                path: vec![PathNode::Key("a".to_string()),],
                value: None,
                or: false,
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
                    PathNode::Index(None),
                    PathNode::Index(Some(1)),
                    PathNode::Key("potato".to_string())
                ],
                value: None,
                or: false,
            },
            pattern
        );
    }
}
