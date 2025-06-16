use match_node::MatchNode;
use serde_json::Value;

use crate::{
    pattern::{pattern_node::PatternNode, Pattern},
    utils::string_utils::wildcard_match,
};

pub mod match_node;

fn match_value(json: &Value, matching_value: &str) -> bool {
    match json {
        Value::Null => wildcard_match("null", matching_value),
        Value::Bool(b) => wildcard_match(&bool::to_string(b), matching_value),
        Value::Number(n) => wildcard_match(n.as_str(), matching_value),
        Value::String(s) => wildcard_match(s, matching_value),
        _ => false, // TODO also match objects and arrays?
    }
}

// The tree is traversed recursively, with two kinds of heads, match heads and start heads.
// The match head is the head of the matching path, and the start head is the head from which match
// paths start.
// IMPORTANT: match order needs to be preserved. Some of the weird dessign decisions taken in this
// function are for that reason. If you plan on refactoring, or modifying keep that in mind.
fn match_internal(
    json: &Value,
    matching_path: &[PatternNode],
    matching_val: Option<&String>,
    path: Vec<MatchNode>,
    or: bool,
    start_head: bool,
) -> Vec<Vec<MatchNode>> {
    let mut result: Vec<Vec<MatchNode>> = Vec::new();

    // Closure to extend the start head path.
    // The start head is extended only by the start head path. It requires no condition to be
    // extended.
    let extend_start_head = |result: &mut Vec<Vec<MatchNode>>, v: &Value, match_node: MatchNode| {
        let mut next_path = path.clone();
        next_path.push(match_node);
        let head_matches = match_internal(v, matching_path, matching_val, next_path, or, true);
        result.extend(head_matches);
    };

    // Closure to extend the match path.
    // When extending the match path, the matching nodes get their first node removed, as it has
    // already been matched
    let extend_match = |result: &mut Vec<Vec<MatchNode>>, v: &Value, match_node: MatchNode| {
        let next_nodes = &matching_path[1..];
        let mut next_path = path.clone();
        next_path.push(match_node);
        let matches = match_internal(v, next_nodes, matching_val, next_path, or, false);
        result.extend(matches);
    };

    // Two possibilities, either there are things left to match, in which case, they need to be
    // matched, and both the start and the match head extended, or the matching path is empty, in
    // which case only values are checked, and the start head extended.
    if matching_path.is_empty() {
        if or || matching_val.map(|m| match_value(json, m)).unwrap_or(true) {
            result.push(path.clone());
        }
        if start_head {
            match json {
                Value::Array(vec) => vec.iter().enumerate().for_each(|(v, i)| {
                    extend_start_head(&mut result, i, MatchNode::new_index(v, false));
                }),
                Value::Object(map) => map.iter().for_each(|(k, v)| {
                    extend_start_head(&mut result, v, MatchNode::new_key(k.to_string(), false));
                }),
                _ => {}
            }
        }
    } else {
        let current_node = &matching_path[0];
        match json {
            Value::Array(json_array) => {
                for (i, v) in json_array.iter().enumerate() {
                    if let PatternNode::Index(index) = current_node {
                        if Some(i) == *index || index.is_none() {
                            extend_match(&mut result, v, MatchNode::new_index(i, true));
                        }
                    }
                    if start_head {
                        extend_start_head(&mut result, v, MatchNode::new_index(i, false));
                    }
                }
            }
            Value::Object(map) => {
                for (k, v) in map.iter() {
                    if let PatternNode::Key(matching_key) = current_node {
                        if wildcard_match(k, matching_key) {
                            extend_match(&mut result, v, MatchNode::new_key(k.to_string(), true));
                        }
                    }
                    if start_head {
                        extend_start_head(&mut result, v, MatchNode::new_key(k.to_string(), false));
                    }
                }
            }
            _ => {
                if or && matching_val.map(|m| match_value(json, m)).unwrap_or(false) {
                    result.push(path);
                }
            }
        }
    }
    result
}

pub fn match_pattern(json: &Value, pattern: &Pattern) -> Vec<Vec<MatchNode>> {
    let mut matches = match_internal(
        json,
        &pattern.path,
        pattern.value.as_ref(),
        vec![],
        pattern.or,
        !pattern.start_at_root,
    );
    matches.dedup();
    matches
}

#[cfg(test)]
pub mod tests {
    use serde_json::json;

    use crate::{
        matcher::{match_pattern, MatchNode},
        pattern::Pattern,
    };

    #[test]
    fn test_complete_path() {
        let pattern = Pattern::parse(".a.b.c").unwrap();

        let json = json!({ "a": { "b": { "c": 42 } } });

        let result = match_pattern(&json, &pattern);

        assert_eq!(
            result,
            vec![vec![
                MatchNode::new_key("a".to_string(), true),
                MatchNode::new_key("b".to_string(), true),
                MatchNode::new_key("c".to_string(), true)
            ]]
        )
    }

    #[test]
    fn test_partial_path() {
        let pattern = Pattern::parse(".b.c").unwrap();

        let json = json!({ "a": { "b": { "c": 42 } } });

        let result = match_pattern(&json, &pattern);

        assert_eq!(
            result,
            vec![vec![
                MatchNode::new_key("a".to_string(), false),
                MatchNode::new_key("b".to_string(), true),
                MatchNode::new_key("c".to_string(), true)
            ]]
        )
    }

    #[test]
    fn test_wildcard_path() {
        let pattern = Pattern::parse(".i*m.c").unwrap();

        let json = json!({ "a": { "item": { "c": 42 } } });

        let result = match_pattern(&json, &pattern);

        assert_eq!(
            result,
            vec![vec![
                MatchNode::new_key("a".to_string(), false),
                MatchNode::new_key("item".to_string(), true),
                MatchNode::new_key("c".to_string(), true)
            ]]
        )
    }

    #[test]
    fn test_value_null() {
        let pattern = Pattern::parse(": null").unwrap();

        let json = json!({"a": "null"});

        let result = match_pattern(&json, &pattern);

        assert_eq!(
            result,
            vec![vec![MatchNode::new_key("a".to_string(), false)]]
        )
    }

    #[test]
    fn test_value_bool() {
        let true_pattern = Pattern::parse(": true").unwrap();
        let false_pattern = Pattern::parse(": false").unwrap();

        let json = json!({"a": true, "b": false});

        let result = match_pattern(&json, &true_pattern);
        assert_eq!(
            result,
            vec![vec![MatchNode::new_key("a".to_string(), false)]]
        );

        let result = match_pattern(&json, &false_pattern);
        assert_eq!(
            result,
            vec![vec![MatchNode::new_key("b".to_string(), false)]]
        );
    }

    #[test]
    fn test_value_number() {
        let pattern = Pattern::parse(": 42").unwrap();

        let json = json!({"a": 42});

        let result = match_pattern(&json, &pattern);

        assert_eq!(
            result,
            vec![vec![MatchNode::new_key("a".to_string(), false)]]
        )
    }

    #[test]
    fn test_value_string() {
        let pattern = Pattern::parse(": hello").unwrap();

        let json = json!({"a": "hello"});

        let result = match_pattern(&json, &pattern);

        assert_eq!(
            result,
            vec![vec![MatchNode::new_key("a".to_string(), false)]]
        )
    }

    #[test]
    fn test_value_and_path() {
        let pattern = Pattern::parse(".a.b.c: 42").unwrap();

        let json = json!({ "a": { "b": { "c": 42 } } });

        let result = match_pattern(&json, &pattern);

        assert_eq!(
            result,
            vec![vec![
                MatchNode::new_key("a".to_string(), true),
                MatchNode::new_key("b".to_string(), true),
                MatchNode::new_key("c".to_string(), true)
            ]]
        )
    }

    #[test]
    fn start_at_root() {
        let pattern = Pattern::parse("$.a").unwrap();

        let json = json!({ "a": 1, "b": { "a": 2}});

        let result = match_pattern(&json, &pattern);

        assert_eq!(
            result,
            vec![vec![MatchNode::new_key("a".to_string(), true)]]
        );
    }
}
