#[derive(Debug, PartialEq, Clone)]
pub enum MatchNode {
    Key(MatchKey),
    Index(MatchIndex),
}

#[derive(Debug, PartialEq, Clone)]
pub struct MatchKey {
    pub key: String,
    pub highlighted: bool,
}

#[derive(Debug, PartialEq, Clone)]
pub struct MatchIndex {
    pub index: usize,
    pub highlighted: bool,
}

impl MatchNode {
    pub fn as_key(&self) -> Option<&MatchKey> {
        match self {
            MatchNode::Key(k) => Some(k),
            _ => None,
        }
    }

    pub fn as_index(&self) -> Option<&MatchIndex> {
        match self {
            MatchNode::Index(i) => Some(i),
            _ => None,
        }
    }

    pub fn is_highlighted(&self) -> bool {
        match self {
            MatchNode::Key(k) => k.highlighted,
            MatchNode::Index(i) => i.highlighted,
        }
    }

    pub fn new_key(key: String, highlighted: bool) -> MatchNode {
        MatchNode::Key(MatchKey { key, highlighted })
    }

    pub fn new_index(index: usize, highlighted: bool) -> MatchNode {
        MatchNode::Index(MatchIndex { index, highlighted })
    }
}

