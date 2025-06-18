
#[derive(Debug, PartialEq, Clone)]
pub enum PatternNode {
    Key(String),
    Index(Option<usize>),
    Recursive(),
}

impl PatternNode {
    pub fn as_key(&self) -> Option<&String> {
        match self {
            PatternNode::Key(k) => Some(k),
            _ => None,
        }
    }

    pub fn as_index(&self) -> Option<&Option<usize>> {
        match self {
            PatternNode::Index(i) => Some(i),
            _ => None,
        }
    }
}

