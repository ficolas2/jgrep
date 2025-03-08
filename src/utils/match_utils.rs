use std::cmp::Ordering;

use itertools::Itertools;

use crate::path_node::PathNode;

pub fn sort_matches(matches: &mut [Vec<PathNode>]) {
    matches.sort_by(|a_match, b_match| {
        let mut ord = Ordering::Equal;
        for zipped in a_match.iter().zip_longest(b_match) {
            match zipped {
                itertools::EitherOrBoth::Both(a_node, b_node) => 
                match (a_node, b_node) {
                    (PathNode::Key(a_key), PathNode::Key(b_key)) => {
                        if a_key != b_key {
                            ord = a_key.cmp(b_key);
                            break;
                        }
                    },
                    (PathNode::Index(a_index), PathNode::Index(b_index)) => {
                        let current_ord = a_index.cmp(b_index);
                        if current_ord != Ordering::Equal {
                            ord = current_ord;
                            break;
                        }
                    },
                    (_, _) => {
                        panic!("Malformed match. A json element cant be both an array and an object")
                    }
                }
                ,
                itertools::EitherOrBoth::Left(_) => {
                    ord = Ordering::Greater;
                    break;
                },
                itertools::EitherOrBoth::Right(_) => {
                    ord = Ordering::Less;
                    break;
                },
            }
        };

        ord
    });
}

#[cfg(test)]
mod tests {
    use crate::path_node::PathNode;
    use super::sort_matches;

    #[test]
    fn test_sort_matches(){
        let mut matches = vec![vec![
            PathNode::Key("b".to_string()),
            PathNode::Key("b".to_string()),
        ],
        vec![
            PathNode::Key("a".to_string()),
            PathNode::Key("a".to_string()),
        ],
        vec![
            PathNode::Key("a".to_string()),
            PathNode::Key("a".to_string()),
            PathNode::Index(Some(1))
        ],
        vec![
            PathNode::Key("a".to_string()),
            PathNode::Key("a".to_string()),
            PathNode::Index(Some(0))
        ],
        vec![
            PathNode::Key("a".to_string()),
        ]];

        let sorted = vec![
            vec![
                PathNode::Key("a".to_string()),
            ],
            vec![
                PathNode::Key("a".to_string()),
                PathNode::Key("a".to_string()),
            ],
            vec![
                PathNode::Key("a".to_string()),
                PathNode::Key("a".to_string()),
                PathNode::Index(Some(0))
            ],
            vec![
                PathNode::Key("a".to_string()),
                PathNode::Key("a".to_string()),
                PathNode::Index(Some(1))
            ],
            vec![
                PathNode::Key("b".to_string()),
                PathNode::Key("b".to_string()),
            ],
        ];
        
        sort_matches(&mut matches);
        assert_eq!(sorted, matches);
    }
}
