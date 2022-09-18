use crate::{atomic_ref::Nullable, AtomicRef, Node, NodeRef, Phase};

#[allow(dead_code)]
#[derive(Debug)]
pub struct OpDesc<T> {
    phase: Phase,
    pending: bool,
    enqueue: bool,
    node: NodeRef<T>,
}

impl<T> OpDesc<T> {
    pub fn new(phase: Phase, pending: bool, enqueue: bool, node: Node<T>) -> Self {
        OpDesc {
            phase,
            pending,
            enqueue,
            node: NodeRef::new(node),
        }
    }

    pub fn finished_enqueue(phase: Phase, node: NodeRef<T>) -> Self {
        OpDesc {
            phase,
            pending: false,
            enqueue: true,
            node,
        }
    }

    pub fn phase(&self) -> Phase {
        self.phase
    }

    pub fn is_enqueue(&self) -> bool {
        self.enqueue
    }

    pub fn is_dequeue(&self) -> bool {
        !self.enqueue
    }

    pub fn is_pending(&self, phase: Phase) -> bool {
        self.pending && self.phase <= phase
    }

    pub fn node(&self) -> NodeRef<T> {
        self.node.clone()
    }
}

impl<T> Default for OpDesc<T> {
    fn default() -> Self {
        Self {
            phase: -1,
            pending: false,
            enqueue: false,
            node: AtomicRef::null(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{atomic_ref::Nullable, OpDesc};

    #[test]
    fn test_default() {
        let op = OpDesc::<()>::default();
        assert_eq!(op.phase(), -1);
        assert_eq!(op.is_pending(-1), false);
        assert_eq!(op.is_enqueue(), false);
        assert_eq!(op.node().is_null(), true);
    }
}
