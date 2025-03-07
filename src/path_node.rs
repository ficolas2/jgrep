
#[derive(Debug, PartialEq)]
pub enum PathNode {
        Key(String),
        Index(Option<usize>),
}

