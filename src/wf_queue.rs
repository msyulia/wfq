use crate::handle::WFQueueHandle;
use crate::{AtomicRef, NodeRef, OpDesc, OpDescRef, Phase};
use core::panic;
use std::iter::repeat_with;
use std::sync::atomic::{AtomicUsize, Ordering};

pub struct WFQueue<T> {
    head: NodeRef<T>,
    tail: NodeRef<T>,
    state: Box<[OpDescRef<T>]>,
    max_threads: usize,
    last_tid: AtomicUsize,
}

impl<T> WFQueue<T> {
    pub fn new(state_length: usize) -> Self {
        if state_length == 0 {
            panic!("State can't be zero sized")
        }
        let mut initial_state = Vec::with_capacity(state_length);
        initial_state.extend(repeat_with(|| AtomicRef::new(OpDesc::default())).take(state_length));
        WFQueue {
            head: AtomicRef::new(None),
            tail: AtomicRef::new(None),
            state: initial_state.into_boxed_slice(),
            max_threads: state_length,
            last_tid: AtomicUsize::new(0),
        }
    }

    pub fn max_phase(&self) -> Phase {
        max_phase(&self.state)
    }

    pub(crate) fn get_state_at_unchecked(&self, index: usize) -> OpDescRef<T> {
        // Safety: Can't construct `Self` with state being zero length without panicking
        // We never give out a handle that has an index bigger than the max allowed index
        let state = unsafe { self.state.get_unchecked(index) };
        state.clone()
    }

    pub(crate) fn set_state_at_unchecked(&self, index: usize, new: OpDesc<T>) {
        // Safety: Can't construct `Self` with state being zero length without panicking
        // We never give out a handle that has an index bigger than the max allowed index
        unsafe { self.state.get_unchecked(index) }.store(new);
    }

    pub(crate) fn get_state_at(&self, index: usize) -> Option<OpDescRef<T>> {
        let state = self.state.get(index).cloned();
        state
    }

    pub(crate) fn help(&self, phase: Phase) {
        for op in self.state.iter() {
            if op.is_enqueue() {
                self.help_enq(phase)
            } else {
                self.help_deq(phase)
            }
        }
    }

    pub(crate) fn help_finish_enqueue(&self) {
        let tail = &*self.tail;
        let next = tail;
        if let Some(node) = next {
            let thread_id = node.enqueue_thread();
            let current_op = self.get_state_at_unchecked(thread_id);
        }
    }

    fn help_deq(&self, phase: Phase) {
        todo!()
    }

    fn help_enq(&self, phase: Phase) {
        todo!()
    }

    pub fn get_handle(&self) -> Option<WFQueueHandle<T>> {
        let last_tid = self.last_tid.load(Ordering::Acquire);
        if last_tid + 1 < self.max_threads {
            let tid = self.last_tid.fetch_add(1, Ordering::Relaxed);
            Some(WFQueueHandle::new(self, tid))
        } else {
            None
        }
    }
}

fn max_phase<T>(state: &[OpDescRef<T>]) -> Phase {
    let mut max = 0;
    for s in state {
        max = if max < s.phase() { s.phase() } else { max };
    }
    max
}

#[cfg(test)]
mod tests {
    // use super::WFQueue;
    // #[test]
    // fn test_new_queue() {
    //     let _ = WFQueue::<()>::new(1);
    //     assert!(true)
    // }
}
