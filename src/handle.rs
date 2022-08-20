use crate::{Node, OpDesc, OpDescRef, Phase, WFQueue};

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
        self.set_current_operation(OpDesc::new(phase, true, true, Some(node)));
        self.queue.help(phase);
        self.queue.help_finish_enqueue(phase);
    }

    pub fn dequeue(&self) -> Option<T> {
        None
    }

    pub fn set_current_operation(&self, new: OpDesc<T>) {
        self.queue.set_state_at_unchecked(self.handle_id, new)
    }

    pub fn get_current_operation(&self) -> OpDescRef<T> {
        self.queue.get_state_at_unchecked(self.handle_id)
    }

    pub fn is_op_pending(&self, phase: Phase) -> bool {
        let state = self.get_current_operation();
        state.is_pending() && state.phase() <= phase
    }
}
