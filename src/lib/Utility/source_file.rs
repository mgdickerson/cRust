use std::fs::File;
use std::io::BufReader;
use std::io::Read;
use std::iter::Peekable;
use std::str::Chars;

use lib::Lexer::token::Token;
use lib::Lexer::*;
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
        src_file: File,
    ) -> Result<SourceFile, Error> {
        let mut src = String::new();
        let result = BufReader::new(src_file)
        .read_to_string(&mut src);

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

    pub fn src_to_iter(&self) -> Peekable<Chars<'_>> {
        self.src.chars().peekable()
    }

    /// Get starting BytePos base on given position within line.
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

    /// Get line number from BytePos for Debugging purposes.
    pub fn line_num(&self, pos: BytePos) -> usize {
        match self.lines.binary_search(&pos) {
            Ok(index) => {
                index
            },
            Err(index) => {
                if index == 0 {
                    0
                } else {
                    index - 1
                }
            }
        }
    }

    pub fn get_src_line(&self, starting_pos: usize) -> & str {
        let starting_byte = self.lines[starting_pos];
        
        if let Some(end_byte) = self.lines.get(starting_pos + 1) {
            &self.src[starting_byte.0 as usize..end_byte.0 as usize]
        } else {
            &self.src[starting_byte.0 as usize..]
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