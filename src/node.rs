use crate::atomic_ref::AtomicRef;
use std::sync::atomic::{AtomicI64, Ordering};

#[allow(dead_code)]
pub struct Node<T> {
    value: T,
    next: AtomicRef<Option<Node<T>>>,
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

    pub fn get_next(&self) -> &Option<Self> {
        &self.next
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

#[cfg(test)]
mod tests {
    use super::Node;

    #[test]
    fn swap_node() {
        let n1 = Node::new(10, -1);
        let n2 = Node::new(20, -1);

        n1.set_next(Some(n2));
    }
}
