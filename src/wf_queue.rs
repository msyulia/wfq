use crate::atomic_ref::Nullable;
use crate::handle::WFQueueHandle;
use crate::{AtomicRef, Node, NodeRef, OpDesc, OpDescRef, Phase};
use core::panic;
use std::iter::repeat_with;
use std::sync::atomic::{AtomicUsize, Ordering};

#[derive(Debug)]
pub struct WFQueue<T> {
    pub(crate) head: AtomicRef<Node<T>>,
    pub(crate) tail: AtomicRef<Node<T>>,
    pub(crate) state: Vec<OpDescRef<T>>,
    max_threads: usize,
    last_tid: AtomicUsize,
}

impl<T> WFQueue<T> {
    pub fn new(state_length: usize) -> Self {
        if state_length == 0 {
            panic!("State can't be zero sized")
        }
        let mut initial_state = Vec::with_capacity(state_length);
        initial_state.extend(repeat_with(|| AtomicRef::default()).take(state_length));
        let initial_node = AtomicRef::new(Node::empty());
        WFQueue {
            head: initial_node.clone(),
            tail: initial_node,
            state: initial_state,
            max_threads: state_length,
            last_tid: AtomicUsize::new(0),
        }
    }

    pub fn max_phase(&self) -> Phase {
        let mut max = 0;
        for s in self.state.iter() {
            max = if max < s.phase() { s.phase() } else { max };
        }
        max
    }

    pub fn get_handle(&self) -> Option<WFQueueHandle<T>> {
        let last_tid = self.last_tid.load(Ordering::Acquire);
        if last_tid < self.max_threads {
            let tid = self.last_tid.fetch_add(1, Ordering::Relaxed);
            Some(WFQueueHandle::new(self, tid))
        } else {
            None
        }
    }

    pub(crate) fn get_operation(&self, index: usize) -> OpDescRef<T> {
        // Safety: Can't construct `Self` with state being zero length without panicking
        // We never give out a handle that has an index bigger than the max allowed index
        let state = unsafe { self.state.get_unchecked(index) };
        state.clone()
    }

    pub(crate) fn swap_operation(&self, index: usize, new: OpDesc<T>) {
        // Safety: Can't construct `Self` with state being zero length without panicking
        // We never give out a handle that has an index bigger than the max allowed index
        unsafe { self.state.get_unchecked(index) }.swap(new);
    }

    pub(crate) fn cas_operation(&self, index: usize, new: OpDesc<T>) -> bool {
        let current_op = self.get_operation(index);
        let new_op = AtomicRef::new(new);
        unsafe { self.state.get_unchecked(index) }
            .compare_and_exchange(current_op, new_op, Ordering::Acquire, Ordering::Relaxed)
            .is_ok()
    }
}

impl<T> IntoIterator for WFQueue<T> {
    type Item = NodeRef<T>;

    type IntoIter = QueueIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        QueueIter {
            current: self.head.clone(),
        }
    }
}

pub struct QueueIter<T> {
    current: NodeRef<T>,
}

impl<T> Iterator for QueueIter<T> {
    type Item = AtomicRef<Node<T>>;

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.current.next();
        if next.is_null() {
            None
        } else {
            self.current = next.clone();
            Some(next)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{Node, OpDesc};

    use super::WFQueue;
    #[test]
    fn test_new_queue() {
        let _ = WFQueue::<()>::new(1);
        let x = vec![1, 2];
        let y = x.iter();
    }

    #[test]
    fn test_swap_opdesc() {
        let q = WFQueue::<()>::new(1);
        let new_op = OpDesc::<()>::new(1, true, true, Node::empty());
        assert_eq!(q.get_operation(0).is_pending(1), false);
        q.swap_operation(0, new_op);
        assert_eq!(q.get_operation(0).is_pending(1), true)
    }

    #[test]
    fn test_cas_opdesc() {
        let q = WFQueue::<()>::new(1);
        let new_op = OpDesc::<()>::new(1, true, true, Node::empty());
        assert_eq!(q.get_operation(0).is_pending(1), false);
        q.cas_operation(0, new_op);
        assert_eq!(q.get_operation(0).is_pending(1), true)
    }

    #[test]
    fn test_enqueue_single_thread() {
        let q = WFQueue::<i64>::new(1);
        let handle = q.get_handle().unwrap();
        handle.enqueue(1);
        assert!(q.tail.value().is_some());
        assert_eq!(q.tail.value().unwrap(), &1);
        assert!(q.head.value().is_some());
        assert_eq!(q.head.value().unwrap(), &1);
    }
}
