use std::sync::atomic::Ordering;

use crate::{atomic_ref::Nullable, AtomicRef, Node, OpDesc, OpDescRef, Phase, WFQueue};

pub type HandleId = usize;

pub struct WFQueueHandle<'a, T> {
    queue: &'a WFQueue<T>,
    handle_id: HandleId,
}

impl<'a, T> WFQueueHandle<'a, T> {
    pub fn new(queue: &'a WFQueue<T>, handle_id: HandleId) -> Self {
        WFQueueHandle { queue, handle_id }
    }
}

impl<'a, T> WFQueueHandle<'a, T> {
    pub fn enqueue(&self, value: T) {
        let phase = self.queue.max_phase() + 1;
        let thread_id = self.handle_id as i64;
        let node = Node::new(value, thread_id);
        self.set_current_operation(OpDesc::new(phase, true, true, node));
        self.help_enq(self.handle_id, phase);
        self.help_finish_enqueue(phase);
    }

    pub fn dequeue(&self) -> Option<T> {
        None
    }

    pub fn set_current_operation(&self, new: OpDesc<T>) {
        self.queue.swap_operation(self.handle_id, new)
    }

    pub fn get_current_operation(&self) -> OpDescRef<T> {
        self.queue.get_operation(self.handle_id)
    }

    pub(crate) fn help(&self, tid: usize, phase: Phase) {
        for op in self.queue.state.iter() {
            if op.is_enqueue() {
                self.help_enq(tid, phase)
            } else {
                self.help_deq(phase)
            }
        }
    }

    fn help_enq(&self, tid: usize, phase: Phase) {
        loop {
            let operation = self.queue.get_operation(tid);
            if !operation.is_pending(phase) {
                break;
            }

            let last = self.queue.tail.clone(); // Tail can never be a null pointer
            let next = last.next();
            if last == self.queue.tail {
                if !next.is_null() {
                    self.help_finish_enqueue(phase)
                } else {
                    if operation.is_pending(phase) {
                        let new = operation.node();
                        if last.compare_and_swap_next(next, new) {
                            self.help_finish_enqueue(phase)
                        }
                    }
                }
            }
        }
    }

    fn help_finish_enqueue(&self, phase: Phase) {
        let last = self.queue.tail.clone();
        let next = last.next();
        if !next.is_null() {
            let tid = next.enqueue_thread() as usize;
            let op_ref = self.queue.get_operation(tid);
            if last == self.queue.tail && op_ref.node() == next {
                let new_op = OpDesc::finished_enqueue(op_ref.phase(), next.clone());
                let _ = self.queue.cas_operation(self.handle_id, new_op);
                let _ = self.queue.tail.compare_and_exchange(
                    last,
                    next,
                    Ordering::Acquire,
                    Ordering::Relaxed,
                );
            }
        }
    }

    fn help_deq(&self, phase: Phase) {
        todo!()
    }
}
