use std::fmt::Write;
use std::fmt::{self, Display, Formatter};
use std::io;

use self::Error::*;

use lib::Lexer::token::{Token, TokenType};
use lib::Utility::display::{MessageBuilder, MessageType};
use lib::Utility::source_file::SourceFile;
use lib::Utility::syntax_position::Span;

/// TODO : Add an Error Handling struct here.
#[derive(Clone, Debug, PartialEq)]
pub enum Error {
    InvalidValueRequest(Token),

    // Lexing Errors
    Msg(String),
    Advance,
    CurrentChar,
    Eof(Token),
    Parse(Token),
    UndefChar(Token),
    UndefOp(Token),
    LexingError(Vec<Error>),

    // Parsing Errors
    NoCodeFound,
    MainNF(Token),
    UxToken(String, Token),
    Redef(Token),
    UndefIdent(Token),
    ParsingError(Vec<Error>),
}

impl Display for Error {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        match *self {
            InvalidValueRequest(ref token) => write!(
                formatter,
                "Invalid value was requested for given token: {:?}",
                token
            ),

            // Lexing Errors
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
            }

            // Parsing Errors
            NoCodeFound => write!(
                formatter,
                "while parsing, neither main nor any other token found"
            ),
            MainNF(ref token) => {
                write!(formatter, "expected main declaraction, found: {:?}", token)
            }
            UxToken(ref string, ref token) => write!(
                formatter,
                "expected {}, found unexpected token: {:?}",
                string, token
            ),
            Redef(ref token) => write!(formatter, "token redefined: {:?}", token),
            UndefIdent(ref token) => write!(formatter, "ident not defined: {:?}", token),
            ParsingError(ref error_collection) => write!(formatter, "Lexing Error"),
        }
    }
}

impl MessageBuilder for Error {
    fn build_message(&self, src_file: &mut SourceFile, output: &mut String) -> fmt::Result {
        match *self {
            InvalidValueRequest(ref token) => self.build_error_message(
                MessageType::Error,
                String::from("InvalidValReq"),
                token.invalid_value(),
                src_file,
                token.get_span(),
                output,
            ),

            // Lexing Errors
            Msg(ref string) => write!(output, "{}", self),
            Advance => write!(output, "{}", self),
            CurrentChar => write!(output, "{}", self),
            Eof(ref token) => self.build_error_message(
                MessageType::Error,
                String::from("EOF"),
                token.get_error_message(),
                src_file,
                token.get_span(),
                output,
            ),
            Parse(ref token) => self.build_error_message(
                MessageType::Error,
                String::from("ParingInt"),
                token.get_error_message(),
                src_file,
                token.get_span(),
                output,
            ),
            UndefChar(ref token) => self.build_error_message(
                MessageType::Error,
                String::from("UndefChar"),
                token.get_error_message(),
                src_file,
                token.get_span(),
                output,
            ),
            UndefOp(ref token) => self.build_error_message(
                MessageType::Error,
                String::from("UndefOp"),
                token.get_error_message(),
                src_file,
                token.get_span(),
                output,
            ),
            LexingError(ref error_collection) => {
                for error in error_collection.iter() {
                    error.build_message(src_file, output);
                }

                write!(output, "")
            }

            // Parsing Errors
            NoCodeFound => self.build_error_message(
                MessageType::Error,
                String::from("NoCodeFound"),
                String::from("No 'main' or other token found"),
                src_file,
                Span::default(),
                output,
            ),
            MainNF(ref token) => self.build_error_message(
                MessageType::Error,
                String::from("MainNF"),
                String::from("Expected main, found unexpected token"),
                src_file,
                token.get_span(),
                output,
            ),
            UxToken(ref string, ref token) => {
                let mut err_mssg = String::new();
                write!(
                    err_mssg,
                    "Expected {} but instead found token: {:?}",
                    string, token
                );
                self.build_error_message(
                    MessageType::Error,
                    String::from("UnexpectedToken"),
                    err_mssg,
                    src_file,
                    token.get_span(),
                    output,
                )
            }
            Redef(ref token) => {
                let mut err_mssg = String::new();
                write!(err_mssg, "Token {:?} redefined.", token);
                self.build_error_message(
                    MessageType::Error,
                    String::from("SymbolRedefinition"),
                    err_mssg,
                    src_file,
                    token.get_span(),
                    output,
                )
            }
            UndefIdent(ref token) => self.build_error_message(
                MessageType::Error,
                String::from("UndefinedIdent"),
                String::from("No definition for ident found"),
                src_file,
                token.get_span(),
                output,
            ),
            ParsingError(ref error_collection) => {
                for error in error_collection.iter() {
                    error.build_message(src_file, output);
                }

                write!(output, "")
            }
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
