pub mod token;

use std::iter::Peekable;
use std::str::Chars;

use self::token::Token;
use self::token::TokenType;
use lib::Utility::error::{Error};
use lib::Utility::syntax_position::{BytePos, Span};

use std;

pub struct Lexer<'lctx,'lxr> {
    char_iter: &'lxr mut Peekable<Chars<'lctx>>,
    current_char: Option<char>,
    buffer: Vec<char>,
    token_collection: Vec<Token>,
    errors: Vec<Error>,
    lo: BytePos,
    hi: BytePos,
    is_comment: bool,
    current_op: Option<TokenType>,
}

impl<'lctx,'lxr> Lexer<'lctx,'lxr> {
    fn new(
        char_iter: &'lxr mut Peekable<Chars<'lctx>>
    ) -> Self {
        Lexer {
            char_iter,
            current_char: None,
            buffer: Vec::new(),
            token_collection: Vec::new(),
            errors: Vec::new(),
            lo: BytePos(0),
            hi: BytePos(0),
            is_comment: false,
            current_op: None,
        }
    }

    pub fn tokenize(
        iter: &'lxr mut Peekable<Chars<'lctx>>
    ) -> Result<Vec<Token>, Vec<Error>> {
        let mut lexer = Lexer::new(iter);
        lexer.collect_tokens();

        if lexer.errors.len() != 0 {
            Err(lexer.errors)
        } else {
            Ok(lexer.token_collection)
        }
    }

    fn advance(&mut self) -> Result<(), Error> {
        match self.char_iter.next() {
            Some(ch) => {
                self.current_char = Some(ch);
                self.hi += 1;
                Ok(())
            }
            None => {
                Err(Error::Eof)
            }
        }
    }

    fn current_char(
        &mut self,
    ) -> Result<char, Error> {
        if let Some(ch) = self.current_char {
            return Ok(ch);
        }

        return Err(Error::Eof);
    }

    fn take_while<F: Fn(char) -> bool>(
        &mut self, 
        pred: F
    ) -> Result<String, Error> {
        let mut buffer = String::new();
        let mut ch = self.current_char()?;
        while pred(ch) {
            buffer.push(ch);
            self.advance();
            ch = self.current_char()?;
        }
        Ok(buffer)
    }

    /// Super simple token builder function, takes necessary information and outputs a Token.
    /// Mostly using this to make span building easier and in a single location.
    fn build_token(
        &mut self,
        token_ty: TokenType,
    ) -> Result<Token, Error> {
        // Build Span, String buf, and get current token type
        let span = Span::new(self.lo, self.hi);
        self.lo = self.hi;  // TODO : Dont think this will be necessary if we add the start to each token.

        Ok(Token::new(token_ty, span))
    }

    fn collect_tokens(
        &mut self
    ) {
        while self.advance() == Ok(()) {
            if let Ok(ch) = self.current_char() {
                let result =
                    match ch {
                        // Alpha characters
                        'a'...'z' | 'A'...'Z' | '_' => self.ident(),
                        
                        // Numerics                
                        '0'...'9' => self.number(),

                        // Non-Generating Tokens
                        '\'' | '\t' | '\r' | '\n' | ' ' => self.build_token(TokenType::None),

                        // Braces and Brackets.
                        '{' => self.build_token(TokenType::LCurly),
                        '[' => self.build_token(TokenType::LSquare),
                        '(' => self.build_token(TokenType::LParen),
                        '}' => self.build_token(TokenType::RCurly),
                        ']' => self.build_token(TokenType::RSquare),
                        ')' => self.build_token(TokenType::RParen),

                        // relOp
                        '=' => self.equal(),
                        '!' | '~' => self.not_equal(),
                        '>' => self.greater_equal(),
                        '<' => self.less_equal(),

                        // mathOp
                        '+' => self.build_token(TokenType::AddOp),
                        '-' => self.build_token(TokenType::SubOp),
                        '*' => self.build_token(TokenType::MulOp),
                        '/' => self.div_or_comment(),

                        // Comment
                        '#' => self.comment(),

                        // Splitters
                        ',' => self.build_token(TokenType::Comma),
                        ';' => self.build_token(TokenType::SemiTermination),
                        '.' => self.build_token(TokenType::ComputationEnd),

                        // Undefined Character Error
                        err => {
                            self.lo = self.hi;  // Need to add tracking even for Errors, otherwise tracking will be off.
                            Err(Error::UndefChar(ch))
                        },
                    };

                match result {
                    Ok(token) => {
                        if token.get_type() != TokenType::None {
                            self.token_collection.push(token);
                        }
                    },
                    Err(error) => self.errors.push(error),
                }
            }
        }
    }

    fn ident(
        &mut self
    ) -> Result<Token, Error> {
        let ident = self.take_while(|ch| ch.is_alphanumeric() || ch == '_')?;
        let token_ty = 
            match ident.as_str() {
                "var" => TokenType::Var,
                "array" => TokenType::Array,
                "function" | "procedure" => TokenType::FuncDecl,
                "main" => TokenType::Computation,
                "let" => TokenType::Assignment,
                "call" => TokenType::FuncCall,
                "if" => TokenType::If,
                "then" => TokenType::Then,
                "else" => TokenType::Else,
                "fi" => TokenType::Fi,
                "while" => TokenType::While,
                "do" => TokenType::Do,
                "od" => TokenType::Od,
                "return" => TokenType::Return,

                _ => TokenType::Ident(ident),
            };

        self.build_token(token_ty)
    }

    fn number(
        &mut self
    ) -> Result<Token, Error> {
        let buffer = self.take_while(|ch| ch.is_numeric())?;

        if let Ok(num) = buffer.parse() {
            self.build_token(TokenType::Number(num))
        } else {
            Err(Error::Parse(buffer))
        }
    }

    fn equal(
        &mut self
    ) -> Result<Token, Error> {
        let mut buffer = String::new();
        buffer.push(self.current_char()?);
        self.advance();

        match self.current_char() {
            Ok('=') => {
                self.build_token(TokenType::EqOp)
            },
            Ok(invalid) => {
                buffer.push(invalid);
                Err(Error::UndefOp(buffer))
            },
            Err(error) => {
                Err(error)
            },
        }
    }

    fn not_equal(
        &mut self
    ) -> Result<Token, Error> {
        let mut buffer = String::new();
        buffer.push(self.current_char()?);
        self.advance();

        match self.current_char() {
            Ok('=') => {
                self.build_token(TokenType::NeqOp)
            },
            Ok(invalid) => {
                buffer.push(invalid);
                Err(Error::UndefOp(buffer))
            },
            Err(error) => {
                Err(error)
            },
        }
    }

    fn greater_equal(
        &mut self
    ) -> Result<Token, Error> {
        if let Some(&ch) = self.char_iter.peek() {
            match ch {
                '=' => {
                    self.advance();
                    self.build_token(TokenType::GeqOp)
                },
                _ => self.build_token(TokenType::GreatOp),
            }
        } else {
            Err(Error::Eof)
        }
    }

    fn less_equal(
        &mut self
    ) -> Result<Token, Error> {
        if let Some(&ch) = self.char_iter.peek() {
            match ch {
                '=' => {
                    self.advance();
                    self.build_token(TokenType::LeqOp)
                },
                _ => self.build_token(TokenType::LessOp),
            }
        } else {
            Err(Error::Eof)
        }
    }

    fn div_or_comment(
        &mut self
    ) -> Result<Token, Error> {
        if let Some(&ch) = self.char_iter.peek() {
            match ch {
                '/' => self.comment(),
                _ => self.build_token(TokenType::DivOp),
            }
        } else {
            Err(Error::Eof)
        }
    }

    fn comment(
        &mut self
    ) -> Result<Token, Error> {
        let buffer = self.take_while(|ch| ch != '\n')?;
        self.build_token(TokenType::Comment(buffer))
    }
}