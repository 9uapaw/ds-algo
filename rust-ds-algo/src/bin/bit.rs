use std::fmt::{Binary, Display, Formatter};
use std::ops::{Shl, Shr, BitOr, ShlAssign, ShrAssign, BitAnd, BitAndAssign, BitOrAssign, BitXor, BitXorAssign};

struct Bits
{
    v: i32,
}

impl Bits
{
    pub fn new(v: i32) -> Self {
        Bits {
            v
        }
    }

    pub fn set_bit(&mut self, pos: i32) {
        self.v = self.v | 1 << pos;
    }

    pub fn clear_bit(&mut self, pos: i32) {
        self.v = self.v & !(1 << pos)
    }

    pub fn modify_bit(&mut self, pos: i32, v: bool) {
        self.v = self.v & !(1 << pos) | -(v as i32) & 1 << pos
    }

    pub fn flip_bit(&mut self, pos: i32) {
        self.v = self.v ^ 1 << pos
    }

    pub fn is_bit_set(&self, pos: i32) -> bool {
        (self.v >> pos & 0b1) > 0
    }

    pub fn count_bit(&self) -> i32 {
        let mut x = self.v;
        let mut count = 0;

        while x != 0 {
            x &= x - 1;
            count += 1;
        }

        count
    }
}

impl Display for Bits
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{} {:#x} {:#b}", self.v, self.v, self.v)
    }
}

fn abs(v: f32) -> f32 {
    const SIGNED: i32 = 1 << 31;

    unsafe {
        let i: i32 = std::mem::transmute(v);
        println!("BIi32 OF VALUE: {:032b}", i);
        println!("BIi32 MASK: {:032b}", !SIGNED);
        println!("RES: {:032b}", i & (!SIGNED));
        std::mem::transmute(i & (!SIGNED))
    }
}

fn main() {
    println!("{}", abs(-1003.0));
    let mut bits = Bits::new(0b110101);
    bits.clear_bit(2);
    println!("{}", bits.count_bit());
    println!("{}", bits);
}
