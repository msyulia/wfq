use std::{
    marker::PhantomData,
    ops::Deref,
    ptr,
    sync::atomic::{AtomicPtr, AtomicUsize, Ordering},
};

pub trait Nullable {
    fn null() -> Self;
    fn is_null(&self) -> bool;
}

#[derive(Debug)]
pub(crate) struct Inner<T> {
    value: T,
    rc: AtomicUsize,
}

impl<T> Inner<T> {
    pub(crate) const fn new(value: T) -> Self {
        Self {
            value,
            rc: AtomicUsize::new(1),
        }
    }

    pub(crate) fn into_ptr(self) -> *mut Self {
        Box::into_raw(Box::new(self))
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
            inner: AtomicPtr::new(inner.into_ptr()),
            _marker: PhantomData,
        }
    }

    pub fn rc(&self) -> usize {
        unsafe { self.as_ref(Ordering::Relaxed) }
            .rc
            .load(Ordering::Relaxed)
    }

    pub fn swap(&self, value: T) -> AtomicRef<T> {
        let new = Inner::new(value);
        let old = self.clone();
        let _ = self.dec_rc(Ordering::Release); // decreasing rc because of the following store
        self.inner.store(new.into_ptr(), Ordering::Release);
        old
    }

    pub fn load(&self, ordering: Ordering) -> &T {
        &unsafe { self.as_ref(ordering) }.value
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
                self.inc_rc(Ordering::AcqRel);
                Ok(AtomicRef::from_raw(ptr))
            }
            Err(ptr) => {
                // We are giving out another reference to `self` therefore we increase the reference count
                self.inc_rc(Ordering::AcqRel);
                Err(AtomicRef::from_raw(ptr))
            }
        }
    }

    pub(crate) unsafe fn as_ref(&self, ordering: Ordering) -> &Inner<T> {
        let ptr = self.inner.load(ordering);
        &*ptr
    }

    fn dec_rc(&self, ordering: Ordering) -> usize {
        unsafe { self.as_ref(Ordering::Relaxed) }
            .rc
            .fetch_sub(1, ordering)
    }

    fn inc_rc(&self, ordering: Ordering) -> usize {
        unsafe { self.as_ref(Ordering::Relaxed) }
            .rc
            .fetch_add(1, ordering)
    }

    fn from_raw(ptr: *mut Inner<T>) -> Self {
        AtomicRef {
            inner: AtomicPtr::new(ptr),
            _marker: PhantomData,
        }
    }
}

impl<T> Eq for AtomicRef<T> {}

impl<T> PartialEq for AtomicRef<T> {
    /// Compares whether the things that both `self` and `other` point to are the same, i.e. whether pointers are equal
    fn eq(&self, other: &Self) -> bool {
        self.inner.load(Ordering::Relaxed) == other.inner.load(Ordering::Relaxed)
    }
}

impl<T> Nullable for AtomicRef<T> {
    fn null() -> Self {
        Self {
            inner: AtomicPtr::new(ptr::null_mut()),
            _marker: PhantomData,
        }
    }

    fn is_null(&self) -> bool {
        self.inner.load(Ordering::Relaxed).is_null()
    }
}

impl<T> Default for AtomicRef<T>
where
    T: Default,
{
    fn default() -> Self {
        let inner = Inner::new(T::default());
        Self {
            inner: AtomicPtr::new(inner.into_ptr()),
            _marker: PhantomData,
        }
    }
}

unsafe impl<T: Sync + Send> Send for AtomicRef<T> {}
unsafe impl<T: Sync + Send> Sync for AtomicRef<T> {}

impl<T> Deref for AtomicRef<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        let temp = unsafe { self.as_ref(Ordering::Relaxed) };
        &temp.value
    }
}

impl<T> Clone for AtomicRef<T> {
    fn clone(&self) -> Self {
        if self.is_null() {
            Self::null()
        } else {
            let ptr = self.inner.load(Ordering::Acquire);

            let old_rc = unsafe { self.as_ref(Ordering::Relaxed) }
                .rc
                .fetch_add(1, Ordering::AcqRel);

            if old_rc >= isize::MAX as usize {
                std::process::abort();
            }

            Self {
                inner: AtomicPtr::new(ptr),
                _marker: PhantomData,
            }
        }
    }
}

impl<T> Drop for AtomicRef<T> {
    fn drop(&mut self) {
        if !self.is_null() {
            let old_rc = unsafe { self.as_ref(Ordering::Relaxed) }
                .rc
                .fetch_sub(1, Ordering::AcqRel);
            if old_rc != 1 {
                return;
            }

            // fence(Ordering::Acquire);
            let ptr = self.inner.load(Ordering::Acquire);
            let _ = unsafe { Box::from_raw(ptr) };
        }
    }
}

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
    fn test_store() {
        let ar = AtomicRef::new(5);
        ar.swap(10);
        assert_eq!(*ar, 10);
    }

    #[test]
    fn test_cas_success() {
        let ar = AtomicRef::new(5);
        assert_eq!(ar.rc(), 1);
        let new_value = AtomicRef::new(10);
        assert_eq!(new_value.rc(), 1);
        let ar_clone = ar.clone();
        assert_eq!(ar_clone.rc(), 2);
        let value = ar
            .compare_and_exchange(ar_clone, new_value, Ordering::Release, Ordering::Relaxed)
            .unwrap();
        assert_eq!(value.rc(), 1);
        assert_eq!(ar.rc(), 1);
        assert_eq!(*value, 5);
        assert_eq!(*ar, 10);
    }

    #[test]
    fn test_cas_failure() {
        let ar1 = AtomicRef::new(5);
        let ar2 = AtomicRef::new(7);
        let ar3 = AtomicRef::new(7);
        let unchanged_value = ar1
            .compare_and_exchange(ar3, ar2, Ordering::Release, Ordering::Relaxed)
            .expect_err("CAS Succeded instead of fail");
        assert_eq!(ar1.rc(), 2);
        assert_eq!(*unchanged_value, 5);
        assert_eq!(*ar1, 5);
    }
}
