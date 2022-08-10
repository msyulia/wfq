use crate::{Node, OpDesc};
use std::iter::repeat_with;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

pub struct WFQueueHandle<'a, T> {
    _queue: &'a WFQueue<T>,
    _handle_id: usize,
}

impl<'a, T> WFQueueHandle<'a, T> {
    pub fn enqueue(&self, _value: T) {
        match self._queue.get_state_at(self._handle_id) {
            Some(_node) => {
                todo!()
            }
            None => todo!(),
        }
    }

    pub fn dequeue(&self) -> Option<T> {
        None
    }
}

pub struct WFQueue<T> {
    _head: Option<Arc<Node<T>>>,
    _tail: Option<Arc<Node<T>>>,
    _state: Box<[Arc<OpDesc<T>>]>,
    _max_threads: usize,
    _last_given_out_tid: AtomicUsize,
}

impl<T> WFQueue<T> {
    pub fn new(state_length: usize) -> Self {
        let mut initial_state = Vec::with_capacity(state_length);
        initial_state.extend(repeat_with(|| Arc::new(OpDesc::default())).take(state_length));
        WFQueue {
            _head: None,
            _tail: None,
            _state: initial_state.into_boxed_slice(),
            _max_threads: state_length,
            _last_given_out_tid: AtomicUsize::new(0),
        }
    }

    pub(crate) fn get_state_at(&self, index: usize) -> Option<Arc<OpDesc<T>>> {
        let state = self._state.get(index).cloned();
        state
    }

    pub fn get_handle(&self) -> WFQueueHandle<T> {
        let tid = self._last_given_out_tid.fetch_add(1, Ordering::SeqCst);
        WFQueueHandle {
            _queue: self,
            _handle_id: tid,
        }
    }
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
