use super::pattern_node::PatternNode;

#[derive(Debug, PartialEq, Clone)]
pub struct Pattern {
    pub path: Vec<PatternNode>,
    pub value: Option<String>,
    pub or: bool,
    pub start_at_root: bool,
}

#[cfg(test)]
mod test {
    use crate::pattern::{parser, pattern_node::{IndexPattern, PatternNode}};

    use super::Pattern;

    #[test]
    fn test_path() {
        let pattern = parser::parse(".a.b.c").unwrap();

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
        let pattern = parser::parse(": true").unwrap();

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
        let pattern = parser::parse(".a.b.c: true").unwrap();

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
        let pattern = parser::parse(".\"a\"").unwrap();

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
        let pattern = parser::parse(r#".[][1]["potato"]"#).unwrap();

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
        let pattern = parser::parse("$.a.b.c").unwrap();

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
        let pattern = parser::parse("$..[]").unwrap();

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
        let pattern = parser::parse(".a..b").unwrap();

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
        let pattern = parser::parse(".a[1,2,3]").unwrap();

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
        let pattern = parser::parse(".a[1:3]").unwrap();

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
        let pattern = parser::parse(".[3:]").unwrap();

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
        let pattern = parser::parse("[-2:]").unwrap();

        assert_eq!(
            Pattern {
                path: vec![
                    PatternNode::Index(IndexPattern::LastN(2)),
                ],
                value: None,
                or: false,
                start_at_root: false,
            },
            pattern
        )
    }
}
