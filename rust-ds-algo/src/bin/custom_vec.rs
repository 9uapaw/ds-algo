#![feature(ptr_internals)]

use std::alloc::handle_alloc_error;
use std::alloc::realloc;
use std::alloc::Layout;
use std::alloc::{alloc, dealloc};
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use std::ptr::Unique;

struct Drain<'a, T> {
    data: PhantomData<&'a mut Vec<T>>,
    raw_iter: RawIter<T>,
}

impl<'a, T> Drain<'a, T> {
    pub fn new(raw_iter: RawIter<T>) -> Self {
        Drain {
            data: PhantomData::default(),
            raw_iter,
        }
    }
}

impl<'a, T> Drop for Drain<'a, T> {
    fn drop(&mut self) {
        for _ in &mut *self {}
    }
}

impl<'a, T> Iterator for Drain<'a, T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.raw_iter.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.raw_iter.size_hint()
    }
}

impl<'a, T> DoubleEndedIterator for Drain<'a, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.raw_iter.next_back()
    }
}

struct RawIter<T> {
    start: *const T,
    end: *const T,
}

impl<T> RawIter<T> {
    pub fn new(raw_arr: &[T]) -> Self {
        RawIter {
            start: raw_arr.as_ptr(),
            end: if raw_arr.len() == 0 {
                raw_arr.as_ptr()
            } else {
                unsafe { raw_arr.as_ptr().add(raw_arr.len()) }
            },
        }
    }

    fn next(&mut self) -> Option<T> {
        if self.start == self.end {
            None
        } else {
            unsafe {
                let val = Some(std::ptr::read(self.start));
                self.start = self.start.add(1);
                val
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = unsafe { self.end.offset_from(self.start) } as usize;

        (len, Some(len))
    }

    fn next_back(&mut self) -> Option<T> {
        if self.start == self.end {
            None
        } else {
            unsafe {
                let val = Some(std::ptr::read(self.end.offset(-1)));
                self.end = self.end.offset(-1);
                val
            }
        }
    }
}

struct RawDynamicArray<T> {
    data: Unique<T>,
    cap: usize,
}

impl<T> RawDynamicArray<T> {
    pub fn new() -> Self {
        RawDynamicArray {
            data: Unique::dangling(),
            cap: 0,
        }
    }

    fn grow(&mut self) {
        let (new_cap, new_layout) = if self.cap == 0 {
            (1, Layout::array::<T>(1).unwrap())
        } else {
            let new_cap = 2 * self.cap;
            (new_cap, Layout::array::<T>(new_cap).unwrap())
        };

        assert!(
            new_layout.size() <= isize::MAX as usize,
            "Allocation too large"
        );

        let new_data_ptr = if self.cap == 0 {
            unsafe { alloc(new_layout) }
        } else {
            let old_layout = Layout::array::<T>(self.cap).unwrap();
            unsafe { realloc(self.data.as_ptr() as *mut u8, old_layout, new_layout.size()) }
        };

        self.data = if let Some(p) = Unique::new(new_data_ptr as *mut T) {
            p
        } else {
            handle_alloc_error(new_layout);
        };

        self.cap = new_cap;
    }
}

impl<T> Drop for RawDynamicArray<T> {
    fn drop(&mut self) {
        let layout = Layout::array::<T>(self.cap).unwrap();
        unsafe {
            dealloc(self.data.as_ptr() as *mut u8, layout);
        }
    }
}

struct IntoIter<T> {
    data: RawDynamicArray<T>,
    iter: RawIter<T>,
}

impl<T> Iterator for IntoIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<T> DoubleEndedIterator for IntoIter<T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.next_back()
    }
}

impl<T> Drop for IntoIter<T> {
    fn drop(&mut self) {
        if self.data.cap != 0 {
            for _ in &mut *self {}
        }
    }
}

struct CustomVec<T> {
    raw: RawDynamicArray<T>,
    len: usize,
}

impl<T> CustomVec<T> {
    pub fn new() -> Self {
        CustomVec {
            raw: RawDynamicArray::new(),
            len: 0,
        }
    }

    pub fn push(&mut self, val: T) {
        if self.len == self.raw.cap {
            self.raw.grow();
        }

        unsafe {
            std::ptr::write(self.next_ptr(), val);
        }

        self.len += 1;
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.len == 0 {
            return None;
        }

        self.len -= 1;
        let val = unsafe { std::ptr::read(self.next_ptr()) };

        Some(val)
    }

    pub fn insert(&mut self, val: T, i: usize) {
        if self.len == self.raw.cap {
            self.raw.grow();
        }

        unsafe {
            std::ptr::copy(
                self.raw.data.as_ptr().add(i),
                self.raw.data.as_ptr().add(i + 1),
                self.len - i,
            );
            std::ptr::write(self.raw.data.as_ptr().add(i), val);
        }

        self.len += 1;
    }

    pub fn into_iter(self) -> IntoIter<T> {
        let len = self.len;
        let data = unsafe { std::ptr::read(&self.raw) };
        let raw_iter = RawIter::new(&self);

        std::mem::forget(self);

        IntoIter {
            data,
            iter: raw_iter,
        }
    }

    pub fn remove(&mut self, i: usize) -> Option<T> {
        if self.len == 0 {
            return None;
        }

        Some(unsafe {
            self.len -= 1;
            let val = std::ptr::read(self.raw.data.as_ptr().add(i));
            std::ptr::copy(
                self.raw.data.as_ptr().add(i + 1),
                self.raw.data.as_ptr().add(i),
                self.len - i,
            );
            val
        })
    }

    pub fn drain(&mut self) -> Drain<T> {
        Drain::new(RawIter::new(&self))
    }

    #[inline]
    unsafe fn next_ptr(&self) -> *mut T {
        self.raw.data.as_ptr().add(self.len)
    }
}

impl<T> Drop for CustomVec<T> {
    fn drop(&mut self) {
        while let Some(_) = self.pop() {}
    }
}

impl<T> Deref for CustomVec<T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        unsafe { std::slice::from_raw_parts(self.raw.data.as_ptr(), self.len) }
    }
}

impl<T> DerefMut for CustomVec<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { std::slice::from_raw_parts_mut(self.raw.data.as_ptr(), self.len) }
    }
}

fn main() {}
