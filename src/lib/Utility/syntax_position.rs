/// Copy of Rust's solution for tracking position within source code.
#[derive(Clone, Copy, Debug, Default, PartialOrd)]
pub struct BytePos(pub u32);

impl BytePos {
    pub fn get_pos(&self) -> u32 {
        self.0
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
    }
}