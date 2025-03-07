use serde_json::Value;

use crate::{path_node::PathNode, pattern::Pattern, string_utils::wildcard_match};

// The strategy for matching is the following recursive BFS:
// - Recursively match the paths. If the path is empty, they all will be matches.
//   - Call match_internal with the advanced path
//   - The head path, signified by the head bool, is the only path capable of creating new
//     matches from the starting path. The others can only continue matches.
// - When the path is fully matched (an empty path or None), then the values can also start being
//   matched
// - When a match is found, it is returned in the result.
fn match_internal(
    json: &Value,
    matching_path: &[PathNode],
    matching_value: &Option<String>,
    path: Vec<PathNode>,
    head: bool,
) -> Vec<Vec<PathNode>> {
    let mut result: Vec<Vec<PathNode>> = Vec::new();

    if head {
        match json {
            Value::Array(vec) => result.extend(vec.iter().enumerate().flat_map(|(index, item)| {
                let mut next_path = path.clone();
                next_path.push(PathNode::Index(Some(index)));

                match_internal(item, matching_path, matching_value, next_path, true)
            })),
            Value::Object(map) => result.extend(map.iter().flat_map(|(k, v)| {
                let mut next_path = path.clone();
                next_path.push(PathNode::Key(k.to_string()));
                match_internal(v, matching_path, matching_value, next_path, true)
            })),
            _ => {}
        }
    }

    if matching_path.is_empty() {
        if let Some(_matching_value) = matching_value {
            match json {
                Value::Null => todo!(),
                Value::Bool(_bool) => todo!(),
                Value::Number(_number) => todo!(),
                Value::String(_string) => todo!(),
                _ => todo!(), // TODO also match objects and arrays?
            };
        } else {
            result.push(path);
        }
    } else {
        let current_node = &matching_path[0];
        let next_nodes = &matching_path[1..];
        match (json, current_node) {
            (Value::Array(json_array), PathNode::Index(index)) => {
                if let Some(index) = index {
                    if let Some(item) = json_array.get(*index) {
                        let mut next_path = path.clone();
                        next_path.push(PathNode::Index(Some(*index)));
                        result.extend(match_internal(
                            item,
                            next_nodes,
                            matching_value,
                            next_path,
                            false,
                        ));
                    }
                } else {
                    result.extend(json_array.iter().enumerate().flat_map(|(i, item)| {
                        let mut next_path = path.clone();
                        next_path.push(PathNode::Index(Some(i)));
                        match_internal(item, next_nodes, matching_value, next_path, false)
                    }));
                }
            }
            (Value::Object(map), PathNode::Key(matching_key)) => result.extend(
                map.iter()
                    .filter(|(k, _)| wildcard_match(k, matching_key))
                    .flat_map(|(k, v)| {
                        let mut next_path = path.clone();
                        next_path.push(PathNode::Key(k.to_string()));
                        match_internal(v, next_nodes, matching_value, next_path, false)
                    }),
            ),
            (_, _) => {}
        }
    }
    result
}

pub fn match_pattern(json: &Value, pattern: &Pattern) -> Vec<Vec<PathNode>> {
    let mut matches = match_internal(json, &pattern.path, &pattern.value, vec![], true);
    matches.dedup();
    matches
}

#[cfg(test)]
pub mod tests {
    use serde_json::json;

    use crate::{matcher::match_pattern, path_node::PathNode, pattern::Pattern};

    #[test]
    fn test_complete_path() {
        let pattern = Pattern::parse(".a.b.c").unwrap();

        let json = json!({ "a": { "b": { "c": 42 } } });

        let result = match_pattern(&json, &pattern);

        assert_eq!(
            result,
            vec![vec![
                PathNode::Key("a".to_string()),
                PathNode::Key("b".to_string()),
                PathNode::Key("c".to_string())
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
                PathNode::Key("a".to_string()),
                PathNode::Key("b".to_string()),
                PathNode::Key("c".to_string())
            ]]
        )
    }
}
