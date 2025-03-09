use std::collections::HashMap;

use serde_json::Value;

use crate::matcher::match_node::MatchNode;

#[derive(Debug)]
pub enum PrinterNode {
        Array {
        vec: HashMap<usize, PrinterNode>,
        highlighted: bool,
    },
        Object {
        map: HashMap<String, PrinterNode>,
        highlighted: bool,
    },
        Value {
        val: String,
        highlighted: bool,
    },
}

impl PrinterNode {
    pub fn new_printed_node_for(value: &Value) -> PrinterNode {
        match value {
            Value::Array(_) => PrinterNode::Array {
                vec: HashMap::new(),
                highlighted: false,
            },
            Value::Object(_) => PrinterNode::Object {
                map: HashMap::new(),
                highlighted: false,
            },
            Value::String(s) => PrinterNode::Value {
                val: format!("\"{}\"", s),
                highlighted: false,
            },
            Value::Number(n) => PrinterNode::Value {
                val: n.to_string(),
                highlighted: false,
            },
            Value::Bool(b) => PrinterNode::Value {
                val: b.to_string(),
                highlighted: false,
            },
            Value::Null => PrinterNode::Value {
                val: "null".to_string(),
                highlighted: false,
            },
        }
    }

    pub fn get_or_insert(&mut self, path_node: &MatchNode, value: &Value) -> &mut PrinterNode {
        match self {
            PrinterNode::Array {
                vec,
                highlighted: _,
            } => {
                let match_i = path_node.as_index().unwrap();
                vec.entry(match_i.index)
                    .or_insert_with(|| Self::new_printed_node_for(value))
            }
            PrinterNode::Object {
                map,
                highlighted: _,
            } => {
                let match_k = path_node.as_key().unwrap();
                map.entry(match_k.key.clone())
                    .or_insert_with(|| Self::new_printed_node_for(value))
            }
            PrinterNode::Value {
                val: _,
                highlighted: _,
            } => panic!("Cannot insert into a value"),
        }
    }

    pub fn insert_full(&mut self, value: &Value) {
        match self {
            PrinterNode::Array { vec, .. } => {
                for (i, v) in value.as_array().unwrap().iter().enumerate() {
                    let mut next_printed_node = Self::new_printed_node_for(v);
                    match v {
                        Value::Array(_) | Value::Object(_) => next_printed_node.insert_full(v),
                        _ => {}
                    }
                    vec.insert(i, next_printed_node);
                }
            }
            PrinterNode::Object { map, .. } => {
                for (k, v) in value.as_object().unwrap() {
                    let mut next_printed_node = Self::new_printed_node_for(v);
                    match v {
                        Value::Array(_) | Value::Object(_) => next_printed_node.insert_full(v),
                        _ => {}
                    }
                    map.insert(k.clone(), next_printed_node);
                }
            }
            PrinterNode::Value { .. } => panic!("Cannot insert into a value"),
        }
    }

    pub fn set_highlight(&mut self, h: bool) {
        match self {
            PrinterNode::Array {
                vec: _,
                highlighted,
            } => *highlighted = h,
            PrinterNode::Object {
                map: _,
                highlighted,
            } => *highlighted = h,
            PrinterNode::Value {
                val: _,
                highlighted,
            } => *highlighted = h,
        }
    }
}

