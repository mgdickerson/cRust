use std::ops::{Add, Sub};

const MAX_LEN: u32 = 0b0111_1111_1111_1111;

/// Copy of Rust's solution for tracking position within source code.
#[derive(Clone, Copy, Debug, Default, PartialOrd, PartialEq, Ord, Eq)]
pub struct BytePos(pub u32);

impl BytePos {
    pub fn get_pos(&self) -> u32 {
        self.0
    }

    pub fn from_usize(pos: usize) -> BytePos {
        BytePos(pos as u32)
    }
}

impl Add for BytePos {
    type Output = BytePos;

    fn add(self, other: BytePos) -> BytePos {
        BytePos(self.0 + other.0)
    }
}

impl Sub for BytePos {
    type Output = BytePos;

    fn sub(self, other: BytePos) -> BytePos {
        // Potentially hazardous behaviour if it produces a negative number.
        BytePos(self.0 - other.0)
    }
}

/// Tracks where Tokens are within the source file to make for 
/// easier debugging message. Following the Rust solution, 
/// only need a start position and length.
#[derive(Default, Copy, Clone, Debug)]
pub struct Span {
    base: u32,
    len: u16,
}

impl Span {
    pub fn new(mut lo: BytePos, mut hi: BytePos) -> Self {
        if lo > hi {
            std::mem::swap(&mut lo, &mut hi);
        }

        let (base, len) = (lo.0, hi.0);

        // Make sure length is not rediculous and base is manageable.
        if len <= MAX_LEN && base <= 0 {
            Span { base, len: len as u16 }
        } else {
            // Unhandled len size, default to len of 0. Other Spans will still be handled
            // correctly, and this will simply not track entire length.
            Span { base, len: 0 }
        }
    }

    pub fn data(&self) -> (BytePos, BytePos) {
        (BytePos(self.base),BytePos(self.len as u32))
    }
}