use core::ops::Index;
use std::{ops::IndexMut, slice::SliceIndex};

#[derive(Debug)]
pub struct RingBuffer<T: Sized, const N: usize> {
    current_write_idx: usize,
    buffer: [T; N],
}

impl<T, const N: usize> Default for RingBuffer<T, N>
where
    T: Sized + Default + Copy,
{
    fn default() -> Self {
        Self {
            current_write_idx: 0,
            buffer: [T::default(); N],
        }
    }
}

impl<T, const N: usize> RingBuffer<T, N>
where
    T: Sized + Copy,
{
    pub fn with_init_val(val: T) -> Self {
        Self {
            current_write_idx: 0,
            buffer: [val; N],
        }
    }
}

impl<T: Sized, const N: usize> Index<isize> for RingBuffer<T, N> {
    type Output = T;
    fn index(&self, index: isize) -> &Self::Output {
        if !(-(N as isize)..(N as isize)).contains(&index) {
            panic!("OOB!");
        }

        let mut buf_idx = index + self.current_write_idx as isize;
        buf_idx -= (buf_idx >= (N as isize)) as isize * (N as isize);
        buf_idx += (buf_idx < 0) as isize * (N as isize);

        return &self.buffer[buf_idx as usize];
    }
}

impl<T: Sized, const N: usize> Index<usize> for RingBuffer<T, N> {
    type Output = T;
    fn index(&self, index: usize) -> &Self::Output {
        if N < index {
            panic!("OOB!");
        }

        let mut buf_idx = index + self.current_write_idx;
        buf_idx -= (buf_idx >= N) as usize * N;
        return &self.buffer[buf_idx];
    }
}

impl<T: Sized, const N: usize> IndexMut<isize> for RingBuffer<T, N> {
    fn index_mut(&mut self, index: isize) -> &mut Self::Output {
        if !(-(N as isize)..(N as isize)).contains(&index) {
            panic!("OOB!");
        }

        let mut buf_idx = index + self.current_write_idx as isize;
        buf_idx -= (buf_idx >= (N as isize)) as isize * (N as isize);
        buf_idx += (buf_idx < 0) as isize * (N as isize);

        return &mut self.buffer[buf_idx as usize];
    }
}

impl<T: Sized, const N: usize> IndexMut<usize> for RingBuffer<T, N> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        if N < index {
            panic!("OOB!");
        }

        let mut buf_idx = index + self.current_write_idx;
        buf_idx -= (buf_idx >= N) as usize * N;
        return &mut self.buffer[buf_idx];
    }
}

impl<T: Sized, const N: usize> RingBuffer<T, N> {
    pub fn push(&mut self, value: T) {
        self.buffer[self.current_write_idx] = value;
        self.current_write_idx += 1;
        if self.current_write_idx >= N {
            self.current_write_idx -= N
        }
    }
}
