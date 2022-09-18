use crate::{
    atomic_ref::{AtomicRef, Nullable},
    NodeRef,
};
use std::sync::atomic::{AtomicI64, Ordering};

#[allow(dead_code)]
#[derive(Debug)]
pub struct Node<T> {
    value: Option<T>,
    next: AtomicRef<Node<T>>,
    enqueue_tid: i64,
    dequeue_tid: AtomicI64,
}

impl<T> Node<T> {
    pub fn new(value: T, thread_id: i64) -> Self {
        Self {
            value: Some(value),
            next: AtomicRef::null(),
            enqueue_tid: thread_id,
            dequeue_tid: AtomicI64::new(-1),
        }
    }

    pub fn empty() -> Self {
        Self {
            value: None,
            next: AtomicRef::null(),
            enqueue_tid: -1,
            dequeue_tid: AtomicI64::new(-1),
        }
    }

    pub fn value(&self) -> Option<&T> {
        self.value.as_ref()
    }

    pub fn enqueue_thread(&self) -> i64 {
        self.enqueue_tid
    }

    pub fn next(&self) -> AtomicRef<Node<T>> {
        self.next.clone()
    }

    pub fn compare_and_swap_next(&self, current: NodeRef<T>, new: NodeRef<T>) -> bool {
        self.next
            .compare_and_exchange(current, new, Ordering::Acquire, Ordering::Relaxed)
            .is_ok()
    }

    pub fn set_next_with_cas_loop(&self, new: NodeRef<T>) {
        let mut done = false;
        let mut current = self.next.clone();
        while !done {
            if let Err(failed) = self.next.compare_and_exchange(
                current.clone(),
                new.clone(),
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

    use crate::NodeRef;

    use super::Node;

    #[test]
    fn swap_node() {
        let n1 = Node::new(10, -1);
        let n2 = NodeRef::new(Node::new(20, -1));

        n1.set_next_with_cas_loop(n2);
        let next = n1.next();
        let v = next.load(Ordering::Relaxed);
        assert_eq!(v.value, Some(20));
    }
}
