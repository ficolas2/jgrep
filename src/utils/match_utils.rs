use std::cmp::Ordering;

use itertools::Itertools;

use crate::pattern::pattern_node::PatternNode;


pub fn sort_matches(matches: &mut [Vec<PatternNode>]) {
    matches.sort_by(|a_match, b_match| {
        let mut ord = Ordering::Equal;
        for zipped in a_match.iter().zip_longest(b_match) {
            match zipped {
                itertools::EitherOrBoth::Both(a_node, b_node) => 
                match (a_node, b_node) {
                    (PatternNode::Key(a_key), PatternNode::Key(b_key)) => {
                        if a_key != b_key {
                            ord = a_key.cmp(b_key);
                            break;
                        }
                    },
                    (PatternNode::Index(a_index), PatternNode::Index(b_index)) => {
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
    use crate::pattern::pattern_node::PatternNode;

    use super::sort_matches;

    #[test]
    fn test_sort_matches(){
        let mut matches = vec![vec![
            PatternNode::Key("b".to_string()),
            PatternNode::Key("b".to_string()),
        ],
        vec![
            PatternNode::Key("a".to_string()),
            PatternNode::Key("a".to_string()),
        ],
        vec![
            PatternNode::Key("a".to_string()),
            PatternNode::Key("a".to_string()),
            PatternNode::Index(Some(1))
        ],
        vec![
            PatternNode::Key("a".to_string()),
            PatternNode::Key("a".to_string()),
            PatternNode::Index(Some(0))
        ],
        vec![
            PatternNode::Key("a".to_string()),
        ]];

        let sorted = vec![
            vec![
                PatternNode::Key("a".to_string()),
            ],
            vec![
                PatternNode::Key("a".to_string()),
                PatternNode::Key("a".to_string()),
            ],
            vec![
                PatternNode::Key("a".to_string()),
                PatternNode::Key("a".to_string()),
                PatternNode::Index(Some(0))
            ],
            vec![
                PatternNode::Key("a".to_string()),
                PatternNode::Key("a".to_string()),
                PatternNode::Index(Some(1))
            ],
            vec![
                PatternNode::Key("b".to_string()),
                PatternNode::Key("b".to_string()),
            ],
        ];
        
        sort_matches(&mut matches);
        assert_eq!(sorted, matches);
    }
}
