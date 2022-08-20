use crate::{AtomicRef, Node, Phase};

pub(crate) struct Inner<T> {
    phase: Phase,
    pending: bool,
    enqueue: bool,
    node: Option<Node<T>>,
}

impl<T> Default for Inner<T> {
    fn default() -> Self {
        Self {
            phase: -1,
            pending: false,
            enqueue: false,
            node: None,
        }
    }
}

#[allow(dead_code)]
pub struct OpDesc<T> {
    inner: AtomicRef<Inner<T>>,
}

impl<T> OpDesc<T> {
    pub fn new(phase: Phase, pending: bool, enqueue: bool, node: Option<Node<T>>) -> Self {
        OpDesc {
            inner: AtomicRef::new(Inner {
                phase,
                pending,
                enqueue,
                node,
            }),
        }
    }

    pub fn phase(&self) -> Phase {
        self.inner.phase
    }

    pub fn is_enqueue(&self) -> bool {
        self.inner.enqueue
    }

    pub fn is_dequeue(&self) -> bool {
        !self.inner.enqueue
    }

    pub fn is_pending(&self) -> bool {
        self.inner.pending
    }
}

impl<T> Default for OpDesc<T> {
    fn default() -> Self {
        Self {
            inner: AtomicRef::new(Inner::default()),
        }
    }
}
