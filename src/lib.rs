mod atomic_ref;
mod handle;
mod node;
mod op_desc;
mod wf_queue;
pub use atomic_ref::AtomicRef;
pub use node::*;
pub use op_desc::*;
pub use wf_queue::*;

type NodeRef<T> = AtomicRef<Option<Node<T>>>;
type OpDescRef<T> = AtomicRef<OpDesc<T>>;
type Phase = i64;
