use std::sync::{atomic::AtomicI64, Arc};

pub struct Node<T> {
    value: T,
    next: Arc<Node<T>>,
    enqueue_tid: i64,
    dequeue_tid: AtomicI64,
}
