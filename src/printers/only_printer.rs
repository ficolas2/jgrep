use std::{cmp::min, io::Write};

use serde_json::Value;

use crate::matcher::match_node::MatchNode;


pub fn print<W: Write>(value: Value, matches: Vec<Vec<MatchNode>>, context: usize, mut writer: W) {
    for path in matches {
        let mut value_to_print = &value;
        let mut path = path.clone();
        path.truncate(path.len() - min(path.len() - 1, context));

        for node in path {
            match node {
                MatchNode::Key(match_k) => { 
                    let k = match_k.key;
                    value_to_print = &value_to_print[&k];
                },
                MatchNode::Index(match_i) => { 
                    let i = match_i.index;
                    value_to_print = &value_to_print[i];
                },
            }
        }
        write!(writer, "{}", value_to_print).unwrap();
        writeln!(writer).unwrap();
    }
}

#[cfg(test)]
mod tests {
    use crate::matcher::match_node::MatchNode;

    #[test]
    fn test_only_printer() {
        let value = serde_json::json!({
            "a": [
                { "c": 0 },
                { "c": 1 },
                { "c": 2 },
                [ { "patatas": "felices" }],
            ],
        });

        let matches = vec![
            vec![
                MatchNode::new_key("a".to_string(), true),
                MatchNode::new_index(0, true),
                MatchNode::new_key("c".to_string(), true),
            ],
            vec![
                MatchNode::new_key("a".to_string(), true),
                MatchNode::new_index(3, true),
                MatchNode::new_index(0, true),
            ],
        ];

        let mut output = Vec::new();
        super::print(value, matches, 0, &mut output);
        let output = String::from_utf8(output).unwrap();

        assert_eq!(output, "0\n{\"patatas\":\"felices\"}\n")
    }
}
