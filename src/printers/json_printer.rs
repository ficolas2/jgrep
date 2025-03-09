use std::io::Write;

use serde_json::Value;

use crate::matcher::match_node::MatchNode;
use colored::Colorize;

use super::printer_node::PrinterNode;

pub fn print<W: Write>(value: Value, mut matches: Vec<Vec<MatchNode>>, context: usize, writer: &mut W) {
    match value {
        Value::Array(_) | Value::Object(_) => {
            let mut printer_node = PrinterNode::new_printed_node_for(&value);
            // printer_node.set_highlight(true);
            for m in matches.iter_mut() {
                add_matches(&mut printer_node, &value, m.clone(), context);
            }
            // sort_matches(&mut matches);
            print_node(None, &printer_node, 0, writer);
            writeln!(writer).unwrap();
        }
        _ => panic!("It shouldn't get here. The json value would be invalid."),
    }
}

// - If the matches are less than the context, the whole value is printed, but the matches are
// printed colored.
// - If the matches are more than the context, the value is traversed, and only the traversal is
// printed
//
// First construct the value to be printed
fn add_matches(
    printer_node: &mut PrinterNode,
    json: &Value,
    mut m: Vec<MatchNode>,
    context: usize,
) {
    let next_node = m.first();
    if let Some(next_node) = next_node {
        if m.len() <= context {
            printer_node.insert_full(json);
        }

        let next_json = get_value(json, next_node);
        let next_printer_node = printer_node.get_or_insert(next_node, next_json);
        if next_node.is_highlighted() {
            next_printer_node.set_highlight(true);
        }
        match next_printer_node {
            PrinterNode::Array { .. } | PrinterNode::Object { .. } => {
                m.remove(0);
                add_matches(next_printer_node, next_json, m, context);
            }
            _ => {}
        }
    } else {
        printer_node.insert_full(json);
    }
}

fn get_value<'a>(value: &'a Value, path_node: &MatchNode) -> &'a Value {
    match (value, path_node) {
        (Value::Array(arr), MatchNode::Index(match_i)) => arr.get(match_i.index).unwrap(),
        (Value::Object(map), MatchNode::Key(match_k)) => map.get(&match_k.key).unwrap(),
        _ => panic!("Invalid path node"),
    }
}

fn print_node<W: Write>(
    node_title: Option<&str>,
    printer_node: &PrinterNode,
    indentation: usize,
    writer: &mut W,
) {
    let indent_str = "  ".repeat(indentation);
    match printer_node {
        PrinterNode::Array { vec, highlighted } => {
            let title = if let Some(node_title) = node_title {
                format!(r#"{}"{}": ["#, indent_str, node_title)
            } else {
                format!("{}[", indent_str)
            };

            let (formatted_title, formatted_end) = if *highlighted {
                (title.red(), format!("{}]", indent_str).red())
            } else {
                (title.normal(), format!("{}]", indent_str).normal())
            };

            writeln!(writer, "{}", formatted_title).unwrap();
            for (&i, node) in vec.iter() {
                print_node(None, node, indentation + 1, writer);
                if i == vec.len() - 1 {
                    writeln!(writer).unwrap();
                } else {
                    writeln!(writer, ",").unwrap();
                }
            }
            write!(writer, "{}", formatted_end).unwrap();
        }
        PrinterNode::Object { map, highlighted } => {
            let title = if let Some(node_title) = node_title {
                format!(r#"{}"{}": {{"#, indent_str, node_title)
            } else {
                format!("{}{{", indent_str)
            };

            let (formatted_title, formatted_end) = if *highlighted {
                (title.red(), format!("{}}}", indent_str).red())
            } else {
                (title.normal(), format!("{}}}", indent_str).normal())
            };

            writeln!(writer, "{}", formatted_title).unwrap();
            for (i, (k, node)) in map.iter().enumerate() {
                print_node(Some(k), node, indentation + 1, writer);
                if i < map.len() - 1 {
                    writeln!(writer, ",").unwrap();
                } else {
                    writeln!(writer).unwrap();
                }
            }
            write!(writer, "{}", formatted_end).unwrap();
        }
        PrinterNode::Value { val, highlighted } => {
            let title = if let Some(node_title) = node_title {
                format!(r#"{}"{}": "#, indent_str, node_title)
            } else {
                indent_str
            };

            if *highlighted {
                write!(writer, "{}{}", title.red(), val).unwrap();
            } else {
                write!(writer, "{}{}", title.normal(), val).unwrap();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;
    use indoc::indoc;

    use crate::{matcher::match_node::MatchNode, printers::json_printer};

    #[test]
    fn test_simple() {
        let json = json!({
            "a": {
                "b": {
                    "c": 2
                },
                "w": {
                    "a": 0,
                    "i": 1
                }
            }
        });

        let matches = vec![
        vec![
            MatchNode::new_key("a".to_string(), false),
            MatchNode::new_key("w".to_string(), false),
            MatchNode::new_key("a".to_string(), false),
        ],
        vec![
            MatchNode::new_key("a".to_string(), false),
            MatchNode::new_key("w".to_string(), false),
            MatchNode::new_key("i".to_string(), false),
        ],
        ];

        let mut output = Vec::new();
        json_printer::print(json, matches, 0, &mut output);

        assert_eq!(
            String::from_utf8(output).unwrap(),
            indoc!(r#"
            {
              "a": {
                "w": {
                  "a": 0,
                  "i": 1
                }
              }
            }
            "#)
        );
    }
}
