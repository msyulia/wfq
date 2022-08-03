extern crate alloc;

pub trait IntoPtr<T> {
    fn into_ptr(self) -> *mut T;
}
