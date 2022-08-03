extern crate alloc;
use alloc::boxed::Box;
use core::{
    marker::PhantomData,
    ptr,
    sync::atomic::{AtomicPtr, Ordering},
};

mod guard;
mod into_ptr;
mod invariant;
pub use guard::AtomicRefGuard;
pub use into_ptr::IntoPtr;
pub use invariant::Invariant;

pub struct AtomicRef<T> {
    ptr: AtomicPtr<T>,
    _marker: PhantomData<Invariant<T>>,
}

impl<T> AtomicRef<T> {
    pub fn new(value: T) -> Self {
        let leaked = Box::leak(Box::new(value));
        Self {
            ptr: AtomicPtr::new(leaked),
            _marker: PhantomData,
        }
    }

    pub const fn empty() -> Self {
        Self {
            ptr: AtomicPtr::new(ptr::null_mut()),
            _marker: PhantomData,
        }
    }

    pub fn load(&self, ordering: Ordering) -> AtomicRefGuard<T> {
        let ptr = self.ptr.load(ordering);
        AtomicRefGuard::new(ptr, false)
    }

    pub fn compare_and_exchange(
        &self,
        current: AtomicRefGuard<T>,
        new: T,
        success: Ordering,
        failure: Ordering,
    ) -> Result<AtomicRefGuard<T>, AtomicRefGuard<T>> {
        let new_value = Box::into_raw(Box::new(new));

        self.ptr
            .compare_exchange(current.into_ptr(), new_value, success, failure)
            .map(|ptr| AtomicRefGuard::new(ptr, true))
            .map_err(|ptr| {
                // Safety: pointer was never used in the CAS operation and was created with `Box::leak`
                drop(unsafe { Box::from_raw(new_value) });
                AtomicRefGuard::new(ptr, false)
            })
    }
}

impl<T> Drop for AtomicRef<T> {
    fn drop(&mut self) {
        let ptr = self.ptr.load(Ordering::Relaxed);
        let _ = unsafe { Box::from_raw(ptr) };
    }
}

#[cfg(test)]
mod tests {
    extern crate alloc;
    use alloc::{
        boxed::Box,
        string::{String, ToString},
        sync::Arc,
        vec,
    };
    use core::sync::atomic::Ordering;

    use super::AtomicRef;

    #[test]
    fn test_box_leak() {
        let leaked = Box::leak(Box::new(5));

        let recreated_box = unsafe { Box::from_raw(leaked) };
        assert_eq!(5, *recreated_box.as_ref());
    }

    #[test]
    fn test_make_load_atomic_ref() {
        let x = 5;
        let ar = AtomicRef::new(x);
        let guard = ar.load(Ordering::Relaxed);

        assert_eq!(5, *guard.as_ref());
    }

    #[test]
    fn test_make_load_borrowed() {
        let x = 5;
        let ar = AtomicRef::new(&x);
        let guard = ar.load(Ordering::Relaxed);
        assert_eq!(5, **(guard.as_ref()));
    }

    #[test]
    fn test_cas_success() {
        let ar = AtomicRef::new(5);
        let guard = ar.load(Ordering::Relaxed);
        let value = ar
            .compare_and_exchange(guard, 10, Ordering::Release, Ordering::Relaxed)
            .expect("CAS Failed instead of succeded");
        assert_eq!(*value.as_ref(), 5);
        let value = ar.load(Ordering::Relaxed);
        assert_eq!(*value.as_ref(), 10);
    }

    #[test]
    fn test_cas_failure() {
        let ar1 = AtomicRef::new(5);
        let ar2 = AtomicRef::new(7);
        let guard = ar2.load(Ordering::Relaxed);
        let unchanged_value_guard = ar1
            .compare_and_exchange(guard, 10, Ordering::Release, Ordering::Relaxed)
            .expect_err("CAS Succeded instead of fail");
        assert_eq!(*unchanged_value_guard.as_ref(), 5);
        let guard = ar1.load(Ordering::Relaxed);
        assert_eq!(*(guard.as_ref()), 5);
    }

    struct TestStruct1 {
        pub name: String,
    }

    // #[test]
    // fn test_complex_type() {
    //     let ar = AtomicRefOption::new(TestStruct1 {
    //         name: "John Doe".to_string(),
    //     });
    //     let mut guard = ar.load(Ordering::Relaxed);
    //     assert_eq!(guard.as_ref().unwrap().name, "John Doe".to_string());
    // }

    // struct TestStruct2<'a> {
    //     pub name: &'a str,
    // }

    // pub fn test_struct_with_lifetimes() {
    //     let ar = AtomicRefOption::new(TestStruct2 { name: "John Doe" });
    //     let guard = ar.load(Ordering::Relaxed);
    //     assert_eq!(guard.as_ref().unwrap().name, "John Doe");
    // }
}
