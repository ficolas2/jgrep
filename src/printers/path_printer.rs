use std::io::Write;

use serde_json::Value;

use crate::path_node::PathNode;

pub fn print<W: Write>(value: Value, matches: Vec<Vec<PathNode>>, mut writer: W) {
    for path in matches {
        let mut value_to_print = &value;
        for node in path {
            match node {
                PathNode::Key(k) => { 
                    value_to_print = &value_to_print[&k];
                    write!(writer, ".{}", k).unwrap() 
                },
                PathNode::Index(Some(i)) => { 
                    value_to_print = &value_to_print[i];
                    write!(writer, "[{}]", i).unwrap() 
                },
                PathNode::Index(None) => {
                    panic!("Print method is not supossed to be called with Path nodes to match")
                }
            }
        }
        write!(writer, ": {}", value_to_print).unwrap();
        writeln!(writer).unwrap();
    }
}

#[cfg(test)]
mod test {
    use crate::path_node::PathNode;

    #[test]
    fn test_printer() {
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
                PathNode::Key("a".to_string()),
                PathNode::Index(Some(0)),
                PathNode::Key("c".to_string()),
            ],
            vec![
                PathNode::Key("a".to_string()),
                PathNode::Index(Some(3)),
                PathNode::Index(Some(0)),
            ],
        ];

        let mut output = Vec::new();
        super::print(value, matches, &mut output);
        let output = String::from_utf8(output).unwrap();

        assert_eq!(output, ".a[0].c: 0\n.a[3][0]: {\"patatas\":\"felices\"}\n")
    }
}
