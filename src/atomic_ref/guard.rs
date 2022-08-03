extern crate alloc;
use core::{marker::PhantomData, ops::Deref};

use alloc::boxed::Box;

use crate::atomic_ref::{IntoPtr, Invariant};

#[derive(Debug)]
pub struct AtomicRefGuard<T> {
    ptr: *mut T,
    should_drop: bool,
    _marker: PhantomData<Invariant<T>>,
}

impl<T> AtomicRefGuard<T> {
    pub(crate) fn new(inner: *mut T, should_drop: bool) -> Self {
        Self {
            ptr: inner,
            should_drop,
            _marker: PhantomData,
        }
    }

    pub fn as_ref(&self) -> &T {
        unsafe { self.ptr.as_ref() }.unwrap()
    }
}

impl<T> Drop for AtomicRefGuard<T> {
    fn drop(&mut self) {
        if self.should_drop {
            drop(unsafe { Box::from_raw(self.ptr) });
        }
    }
}

impl<T> IntoPtr<T> for AtomicRefGuard<T> {
    #[inline(always)]
    fn into_ptr(self) -> *mut T {
        self.ptr
    }
}

// impl<T> Deref for AtomicRefGuard<T> {
//     type Target = T;

//     fn deref(&self) -> &Self::Target {
//         // Safety: We don't provide a way of modifying the pointer other than atomic modifications
//         unsafe { &*self.ptr }
//     }
// }
