use crate::{atomic_ref::AtomicRef, NodeRef};
use std::sync::atomic::{AtomicI64, Ordering};

#[allow(dead_code)]
pub struct Node<T> {
    value: T,
    next: NodeRef<T>,
    enqueue_tid: i64,
    dequeue_tid: AtomicI64,
}

impl<T> Node<T> {
    pub fn new(value: T, thread_id: i64) -> Self {
        Node {
            value,
            next: AtomicRef::new(None),
            enqueue_tid: thread_id,
            dequeue_tid: AtomicI64::new(-1),
        }
    }

    pub fn enqueue_thread(&self) -> i64 {
        self.enqueue_tid
    }

    pub fn next(&self) -> NodeRef<T> {
        self.next.clone()
    }

    pub fn set_next(&self, new: Option<Node<T>>) {
        let mut done = false;
        let mut current = self.next.clone();
        let new_node = AtomicRef::new(new);
        while !done {
            if let Err(failed) = self.next.compare_and_exchange(
                current.clone(),
                new_node.clone(),
                Ordering::Acquire,
                Ordering::Relaxed,
            ) {
                current = failed;
            } else {
                done = true;
            }
        }
    }
}

impl<T> PartialEq for Node<T> {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self, other)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::Ordering;

    use super::Node;

    #[test]
    fn swap_node() {
        let n1 = Node::new(10, -1);
        let n2 = Node::new(20, -1);

        n1.set_next(Some(n2));
        let next = n1.next();
        assert!(next.is_some());
        if let Some(v) = next.load(Ordering::Relaxed) {
            assert_eq!(v.value, 20)
        }
    }
}
