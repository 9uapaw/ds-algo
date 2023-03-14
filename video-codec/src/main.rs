use std::cmp::{max, min};
use bitvec::bitvec;
use bitvec::prelude::BitVec;

/// Reimplementation of https://blog.tempus-ex.com/hello-video-codec/

pub type Pixel = u16;

pub struct Plane<T> {
    pub data: T,
    pub width: usize,
    pub height: usize,
    pub sample_stride: usize,
    pub row_stride: usize,
}

pub fn fixed_prediction(a: Pixel, b: Pixel, c: Pixel) -> i32 {
    let min_a_b = min(a, b);
    let max_a_b = max(a, b);

    if c >= max_a_b {
        min_a_b as i32
    } else if c <= min_a_b {
        max_a_b as i32
    } else {
        (a + b - c) as i32
    }
}

pub fn rice_coder(k: u32, num: u64, buf: &mut BitVec) {
    let m = 2_u64.pow(k);
    let q = (num / m);

}

fn main() {
    println!("{:#b}", 5);
    println!("{:#b}", 5 >> 2);
}
