use super::syntax_position::{BytePos};
use super::error::Error;

pub struct SourceFile {
    pub name: String,
    pub src: String,
    pub start_pos: BytePos,
    pub end_pos: BytePos,
    pub lines: Vec<BytePos>,
}

impl SourceFile {
    pub fn new(
        name: String,
        src: String,
    ) -> Result<SourceFile, Error> {
        // All files will start at position 0.
        let start_pos = BytePos(0);
        let end_pos = src.len();

        if end_pos > u32::max_value() as usize {
            return Err(Error::Msg(String::from("ending position exceeds max u32 value")));
        }

        let lines = analyze_source_file(&src, start_pos.clone());

        Ok(SourceFile {
            name,
            src,
            start_pos,
            end_pos: BytePos::from_usize(end_pos),
            lines,
        })
    }

    pub fn line_begin_pos(&self, pos: BytePos) -> BytePos {
        match self.lines.binary_search(&pos) {
            Ok(index) => {
                self.lines[index]
            },
            Err(index) => {
                if index == 0 {
                    self.lines[index]
                } else {
                    self.lines[index - 1]
                }
            },
        }
    }
}

fn analyze_source_file(
    src: &str,
    start_pos: BytePos
) -> Vec<BytePos> {
    let mut i = 0;
    let src_chars = src.chars();
    let mut lines = Vec::default();

    for c in src_chars {
        let pos = BytePos::from_usize(i);

        match c {
            '\n' => {
                lines.push(pos + BytePos(1));
            },
            _ => {},
        }

        i += 1;
    }

    lines
}