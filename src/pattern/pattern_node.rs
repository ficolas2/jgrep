#[derive(Debug, PartialEq, Clone)]
pub enum IndexPattern {
    Range(usize, Option<usize>),
    All,
    List(Vec<usize>),
    LastN(usize),
}

#[derive(Debug, PartialEq, Clone)]
pub enum PatternNode {
    Key(String),
    Index(IndexPattern),
    Recursive(),
}

impl PatternNode {
    pub fn as_key(&self) -> Option<&String> {
        match self {
            PatternNode::Key(k) => Some(k),
            _ => None,
        }
    }

    pub fn as_index(&self) -> Option<&IndexPattern> {
        match self {
            PatternNode::Index(i) => Some(i),
            _ => None,
        }
    }
}

impl IndexPattern {
    pub fn matches(&self, index: usize, len: usize) -> bool {
        match self {
            IndexPattern::Range(start, end) => {
                if let Some(end) = *end {
                    index >= *start && index < end
                } else {
                    index >= *start
                }
            },
            IndexPattern::All => true,
            IndexPattern::List(indices) => indices.contains(&index),
            IndexPattern::LastN(n) => index >= len - *n,
        }
    }
}
