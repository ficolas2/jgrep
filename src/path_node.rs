
#[derive(Debug, PartialEq, Clone)]
pub enum PathNode {
    Key(String),
    Index(Option<usize>),
}

