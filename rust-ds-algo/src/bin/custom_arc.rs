#![feature(ptr_internals)]

use std::ops::Deref;
use std::ptr::Unique;
use std::sync::atomic::{fence, AtomicUsize, Ordering};

pub struct CustomArc<T> {
    inner: Unique<InnerArc<T>>,
}

impl<T> CustomArc<T> {
    pub fn new(data: T) -> Self {
        let boxed = Box::new(InnerArc {
            count: AtomicUsize::new(1),
            data,
        });
        CustomArc {
            inner: Unique::new(Box::into_raw(boxed)).unwrap(),
        }
    }
}

struct InnerArc<T> {
    count: AtomicUsize,
    data: T,
}

unsafe impl<T: Send + Sync> Send for CustomArc<T> {}
unsafe impl<T: Send + Sync> Sync for CustomArc<T> {}

impl<T> Deref for CustomArc<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &(*self.inner.as_ptr()).data }
    }
}

impl<T> Clone for CustomArc<T> {
    fn clone(&self) -> Self {
        let inner = unsafe { &(*self.inner.as_ptr()) };
        let old_rc = inner.count.fetch_add(1, Ordering::Relaxed);

        if old_rc as isize > isize::MAX {
            std::process::abort();
        }

        CustomArc {
            inner: self.inner.clone(),
        }
    }
}

impl<T> Drop for CustomArc<T> {
    fn drop(&mut self) {
        let inner = unsafe { &(*self.inner.as_ptr()) };

        if inner.count.fetch_sub(1, Ordering::Release) != 1 {
            return;
        }

        fence(Ordering::Acquire);

        unsafe { Box::from_raw(self.inner.as_ptr()) };
    }
}

fn main() {}
