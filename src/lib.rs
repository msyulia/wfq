mod atomic_ref;
mod handle;
mod node;
mod op_desc;
mod wf_queue;
pub use atomic_ref::AtomicRef;
pub use node::*;
pub use op_desc::*;
pub use wf_queue::*;

type NodeRef<T> = AtomicRef<Node<T>>;
type OpDescRef<T> = AtomicRef<OpDesc<T>>;
type Phase = i64;

#[cfg(test)]
mod tests {

    #[inline(always)]
    fn foo() {
        println!("Weirdness")
    }

    fn execute<F: FnOnce() -> ()>(fun: F) {
        fun()
    }
    #[test]
    pub fn test_inline_function_pointer_weirdness() {
        let v = vec![1, 2, 3];

        let mut v2 = Vec::<i32>::with_capacity(3);
        v2[0] = 1;
        v2[1] = 2;
        v2[2] = 3;
    }
}
