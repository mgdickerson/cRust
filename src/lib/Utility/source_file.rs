use super::syntax_position::{BytePos};

pub struct SourceFile {
    pub name: String,
    pub src: String,
    pub start_pos: BytePos,
    pub end_pos: BytePos,
    pub lines: Vec<BytePos>,
}

impl SourceFile {}