use std::{
    marker::PhantomData,
    ops::Deref,
    ptr,
    sync::atomic::{fence, AtomicPtr, AtomicUsize, Ordering},
    sync::Arc,
};

#[derive(Debug)]
pub(crate) struct Inner<T> {
    value: T,
    rc: AtomicUsize,
}

impl<T> Inner<T> {
    pub(crate) fn new(value: T) -> Self {
        Self {
            value,
            rc: AtomicUsize::new(1),
        }
    }

    pub(crate) fn to_ptr(self) -> *mut Self {
        Box::into_raw(Box::new(self))
    }
}

impl<T> Drop for Inner<T> {
    fn drop(&mut self) {
        println!("Dropping inner");
    }
}

#[derive(Debug)]
pub struct AtomicRef<T> {
    inner: AtomicPtr<Inner<T>>,
    _marker: PhantomData<Inner<T>>,
}

impl<T> AtomicRef<T> {
    pub fn new(value: T) -> Self {
        let inner = Inner::new(value);
        Self {
            inner: AtomicPtr::new(inner.to_ptr()),
            _marker: PhantomData,
        }
    }

    pub(crate) fn from_raw(ptr: *mut Inner<T>) -> Self {
        AtomicRef {
            inner: AtomicPtr::new(ptr),
            _marker: PhantomData,
        }
    }

    pub(crate) fn to_raw(&self) -> *mut Inner<T> {
        self.inner.load(Ordering::Relaxed)
    }

    pub(crate) unsafe fn rc(&self) -> usize {
        unsafe { self.as_ref() }.rc.load(Ordering::Relaxed)
    }

    pub(crate) unsafe fn dec_rc(&self, ordering: Ordering) -> usize {
        unsafe { self.as_ref() }.rc.fetch_sub(1, ordering)
    }

    pub(crate) unsafe fn inc_rc(&self, ordering: Ordering) -> usize {
        unsafe { self.as_ref() }.rc.fetch_add(1, ordering)
    }

    pub(crate) unsafe fn as_ref(&self) -> &Inner<T> {
        let ptr = self.inner.load(Ordering::Relaxed);
        &*ptr
    }

    pub fn compare_and_exchange(
        &self,
        current: AtomicRef<T>,
        new: AtomicRef<T>,
        success: Ordering,
        failure: Ordering,
    ) -> Result<AtomicRef<T>, AtomicRef<T>> {
        match self.inner.compare_exchange(
            current.inner.load(Ordering::Relaxed),
            new.inner.load(Ordering::Relaxed),
            success,
            failure,
        ) {
            Ok(ptr) => {
                // New is now being referenced in at least two places `self` and `new`
                // need to increase reference count
                unsafe { self.inc_rc(Ordering::Release) };
                // No need to decrease the old pointers reference count because we yes it is being reference in one place less
                // but we return it in the result therefore the operations cancel out
                Ok(AtomicRef::from_raw(ptr))
            }
            Err(ptr) => Err(AtomicRef::from_raw(ptr)),
        }
    }
}

impl<T> Deref for AtomicRef<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        let temp = unsafe { self.as_ref() };
        &temp.value
    }
}

impl<T> Clone for AtomicRef<T> {
    fn clone(&self) -> Self {
        let ptr = self.inner.load(Ordering::Acquire);

        let old_rc = unsafe { self.as_ref() }.rc.fetch_add(1, Ordering::AcqRel);

        if old_rc >= isize::MAX as usize {
            std::process::abort();
        }

        Self {
            inner: AtomicPtr::new(ptr),
            _marker: PhantomData,
        }
    }
}

impl<T> Drop for AtomicRef<T> {
    fn drop(&mut self) {
        let old_rc = unsafe { self.as_ref() }.rc.fetch_sub(1, Ordering::AcqRel);
        if old_rc != 1 {
            return;
        }

        // fence(Ordering::Acquire);
        let ptr = self.inner.load(Ordering::Acquire);
        let _ = unsafe { Box::from_raw(ptr) };
    }
}

unsafe impl<T: Sync + Send> Send for AtomicRef<T> {}
unsafe impl<T: Sync + Send> Sync for AtomicRef<T> {}

#[cfg(test)]
mod tests {
    use std::sync::atomic::Ordering;

    use super::AtomicRef;

    #[test]
    fn test_make_load_atomic_ref() {
        let x = 5;
        let ar = AtomicRef::new(x);
        let new = *ar;

        assert_eq!(5, new);
    }

    #[test]
    fn test_make_load_borrowed() {
        let x = 5;
        let ar = AtomicRef::new(&x);
        let new = *ar;
        assert_eq!(5, *new);
    }

    #[test]
    fn test_cas_success() {
        let ar = AtomicRef::new(5);
        assert_eq!(unsafe { ar.rc() }, 1);
        let new_value = AtomicRef::new(10);
        assert_eq!(unsafe { new_value.rc() }, 1);
        let ar_clone = ar.clone();
        assert_eq!(unsafe { ar_clone.rc() }, 2);
        let value = ar
            .compare_and_exchange(ar_clone, new_value, Ordering::Release, Ordering::Relaxed)
            .unwrap();
        assert_eq!(unsafe { value.rc() }, 1);
        assert_eq!(unsafe { ar.rc() }, 1);
        assert_eq!(*value, 5);
        assert_eq!(*ar, 10);
    }

    #[test]
    fn test_cas_failure() {
        let ar1 = AtomicRef::new(5);
        let ar2 = AtomicRef::new(7);
        let unchanged_value = ar1
            .compare_and_exchange(ar1.clone(), ar2, Ordering::Release, Ordering::Relaxed)
            .expect_err("CAS Succeded instead of fail");
        assert_eq!(*unchanged_value, 5);
        assert_eq!(*ar1, 5);
    }
}
