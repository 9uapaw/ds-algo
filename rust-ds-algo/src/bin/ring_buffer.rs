use std::mem::MaybeUninit;
use std::ptr::NonNull;
use std::sync::atomic::{AtomicUsize, Ordering};

pub type Result<T> = std::result::Result<T, RingBufferError>;

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq)]
pub enum RingBufferError {
    Full,
    Empty,
    PointerUpdateError,
}

struct RingBuffer<T, const N: usize> {
    buf: [Option<T>; N],
    read: AtomicUsize,
    write: AtomicUsize,
}

impl<T, const N: usize> RingBuffer<T, N> {
    const INIT: Option<T> = None;

    pub fn new() -> Self {
        RingBuffer {
            buf: [Self::INIT; N],
            read: AtomicUsize::new(0),
            write: AtomicUsize::new(0),
        }
    }

    pub fn push(&mut self, value: T) -> Result<()> {
        if self.full() {
            return Err(RingBufferError::Full);
        }

        self.buf[self
            .write
            .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |w| Some((w + 1) % N))
            .map_err(|_| RingBufferError::PointerUpdateError)?]
        .replace(value);

        Ok(())
    }

    pub fn pop(&mut self) -> Result<T> {
        if self.empty() {
            return Err(RingBufferError::Empty);
        }

        self.buf[self
            .read
            .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |w| Some((w + 1) % N))
            .map_err(|_| RingBufferError::PointerUpdateError)?]
        .take()
        .ok_or(RingBufferError::Empty)
    }

    pub fn full(&self) -> bool {
        self.buf[N - 1].is_some() && self.buf[0].is_some()
    }

    pub fn empty(&self) -> bool {
        self.buf[self.read.load(Ordering::SeqCst)].is_none()
    }
}


pub fn main() {
}

#[cfg(test)]
mod tests {
    use crate::{RingBuffer, RingBufferError};

    const BUFFER_SIZE: usize = 4;

    #[derive(Debug, Ord, PartialOrd, Eq, PartialEq)]
    struct Test {
        test: usize,
    }

    #[test]
    fn test_push_pop() {
       let mut ring_buffer = RingBuffer::<Test, BUFFER_SIZE>::new();
        let err_empty = ring_buffer.pop();
        assert_eq!(err_empty, Err(RingBufferError::Empty));

        let push_res = ring_buffer.push(Test{test: 0});
        let pop_res = ring_buffer.pop();
        match pop_res {
            Ok(Test{test}) => assert_eq!(test, 0),
            _ => panic!("Pop returned {:#?}", pop_res)
        }
        let err_empty = ring_buffer.pop();
        assert_eq!(err_empty, Err(RingBufferError::Empty));

        for i in 0..BUFFER_SIZE {
           ring_buffer.push(Test{test: i});
        }

        let push_res = ring_buffer.push(Test{test: 5});
        match push_res {
            Err(RingBufferError::Full) => (),
            _ => panic!("Pop returned {:#?}", pop_res)
        }
        let pop_res = ring_buffer.pop();
        match pop_res {
            Ok(Test{test}) => assert_eq!(test, 0),
            _ => panic!("Pop returned {:#?}", pop_res)
        }
        let pop_res = ring_buffer.pop();
        match pop_res {
            Ok(Test{test}) => assert_eq!(test, 1),
            _ => panic!("Pop returned {:#?}", pop_res)
        }



   }
}
