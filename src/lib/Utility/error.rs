use std::fmt::{self, Display, Formatter};
use std::io;
use std::fmt::Write;

use self::Error::*;

use lib::Lexer::token::{Token, TokenType};
use lib::Utility::display::{MessageBuilder, MessageType};
use lib::Utility::source_file::SourceFile;


/// TODO : Add an Error Handling struct here.
#[derive(Clone,Debug,PartialEq)]
pub enum Error {
    Msg(String),
    Advance,
    CurrentChar,
    Eof(Token),
    Parse(Token),
    UndefChar(Token),
    UndefOp(Token),
    LexingError(Vec<Error>),
}

impl Display for Error {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        match *self {
            Msg(ref string) => write!(formatter, "{}", string),
            Advance => write!(formatter, "Expected next token, found EOF"),
            CurrentChar => write!(formatter, "Expected current token, found EOF"),
            Eof(ref token) => write!(formatter, "end of file: {:?}", token),
            Parse(ref token) => write!(formatter, "unable to parse string {:?}", token),
            UndefChar(ref token) => write!(formatter, "unsupported character used: {:?}", token),
            UndefOp(ref token) => write!(formatter, "undefined operation: {:?}", token),
            LexingError(ref error_collection) => {
                // TODO : Add lexing error reporting
                write!(formatter, "Lexing Error")
            },
        }
    }
}

impl MessageBuilder for Error {
    fn build_message(&self, src_file: &mut SourceFile, output: &mut String) -> fmt::Result {
        match *self {
            Msg(ref string) => write!(output, "{}", self),
            Advance => write!(output, "{}", self),
            CurrentChar => write!(output, "{}", self),
            Eof(ref token) => self.build_error_message(MessageType::Error, String::from("EOF"), token.get_error_message(), src_file, token.get_span(), output),
            Parse(ref token) => self.build_error_message(MessageType::Error, String::from("ParingInt"), token.get_error_message(), src_file, token.get_span(), output),
            UndefChar(ref token) => self.build_error_message(MessageType::Error, String::from("UndefChar"), token.get_error_message(), src_file, token.get_span(), output),
            UndefOp(ref token) => self.build_error_message(MessageType::Error, String::from("UndefOp"), token.get_error_message(), src_file, token.get_span(), output),
            LexingError(ref error_collection) => {
                for error in error_collection.iter() {
                    error.build_message(src_file, output);
                }

                write!(output, "")
            },
        }
    }
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Msg(error.to_string())
    }
}

impl<'a> From<&'a Error> for Error {
    fn from(error: &'a Error) -> Self {
        error.clone()
    }
}