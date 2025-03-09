
#[derive(Debug, PartialEq, Clone)]
pub enum PathNode {
    Key(String),
    Index(Option<usize>),
}

impl PathNode {
    pub fn as_key(&self) -> Option<&String> {
        match self {
            PathNode::Key(k) => Some(k),
            _ => None,
        }
    }

    pub fn as_index(&self) -> Option<&Option<usize>> {
        match self {
            PathNode::Index(i) => Some(i),
            _ => None,
        }
    }
}

