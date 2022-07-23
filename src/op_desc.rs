use crate::Node;

pub struct OpDesc<T> {
    phase: i64,
    pending: bool,
    enqueue: bool,
    node: Option<Node<T>>,
}

impl<T> OpDesc<T> {
    pub fn new(phase: i64, pending: bool, enqueue: bool, node: Option<Node<T>>) -> Self {
        OpDesc {
            phase,
            pending,
            enqueue,
            node,
        }
    }
}

impl<T> Default for OpDesc<T> {
    fn default() -> Self {
        Self {
            phase: -1,
            pending: false,
            enqueue: true,
            node: None,
        }
    }
}
