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

// The strategy for matching is the following recursive BFS:
// - Recursively match the paths. If the path is empty, they all will be matches.
//   - Call match_internal with the advanced path
//   - The head path, signified by the head bool, is the only path capable of creating new
//     matches from the starting path. The others can only continue matches.
// - When the path is fully matched (an empty path or None), then the values can also start being
//   matched
// - When a match is found, it is returned in the result.
//
// IMPORTANT: match order needs to be preserved. Some of the weird dessign decisions taken in this
// function are for that reason. If you plan on refactoring, or modifying keep that in mind.
fn match_internal(
    json: &Value,
    matching_path: &[PatternNode],
    matching_val: Option<&String>,
    path: Vec<MatchNode>,
    or: bool,
    head: bool,
) -> Vec<Vec<MatchNode>> {
    let mut result: Vec<Vec<MatchNode>> = Vec::new();

    if matching_path.is_empty() {
        if or || matching_val.map(|m| match_value(json, m)).unwrap_or(true) {
            result.push(path.clone());
        }
        if head {
            match json {
                Value::Array(vec) => {
                    result.extend(vec.iter().enumerate().flat_map(|(index, item)| {
                        let mut next_path = path.clone();
                        next_path.push(MatchNode::new_index(index, false));

                        match_internal(item, matching_path, matching_val, next_path, or, true)
                    }))
                }
                Value::Object(map) => result.extend(map.iter().flat_map(|(k, v)| {
                    let mut next_path = path.clone();
                    next_path.push(MatchNode::new_key(k.to_string(), false));
                    match_internal(v, matching_path, matching_val, next_path, or, true)
                })),
                _ => {}
            }
        }
    } else {
        let current_node = &matching_path[0];
        let next_nodes = &matching_path[1..];
        match json {
            Value::Array(json_array) => {
                for (i, v) in json_array.iter().enumerate() {
                    if let PatternNode::Index(index) = current_node {
                        if Some(i) == *index || index.is_none() {
                            let mut next_path = path.clone();
                            next_path.push(MatchNode::new_index(i, true));
                            let matches =
                                match_internal(v, next_nodes, matching_val, next_path, or, false);
                            result.extend(matches);
                        }
                    }
                    if head {
                        let mut next_path = path.clone();
                        next_path.push(MatchNode::new_index(i, false));
                        let head_matches =
                            match_internal(v, matching_path, matching_val, next_path, or, true);
                        result.extend(head_matches);
                    }
                }
            }
            Value::Object(map) => {
                for (k, v) in map.iter() {
                    if let PatternNode::Key(matching_key) = current_node {
                        if wildcard_match(k, matching_key) {
                            let mut next_path = path.clone();
                            next_path.push(MatchNode::new_key(k.to_string(), true));
                            let matches =
                                match_internal(v, next_nodes, matching_val, next_path, or, false);
                            result.extend(matches);
                        }
                    }
                    if head {
                        let mut next_path = path.clone();
                        next_path.push(MatchNode::new_key(k.to_string(), false));
                        let head_matches =
                            match_internal(v, matching_path, matching_val, next_path, or, true);
                        result.extend(head_matches);
                    }
                }
            }
            _ => {
                if or && matching_val.map(|m| match_value(json, m)).unwrap_or(false) {
                    result.push(path.clone());
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
        true,
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
}
