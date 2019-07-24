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

// pub fn tokenize(iter: &mut Peekable<Chars<'_>>) -> Result<Vec<Token>, Vec<Error>> {

// }

// //  As per Fabian's suggestion: use this function to request a token,
// //then pass the token back. This function will take a string version of
// //the code files, grab tokens, then throw back a single token when found.

// // TODO : Potential optimization or reworks?

// //This seems to work in separating tokens, but may need revisiting for better
// //clarity of tokens or perhaps consolidation, we shall see.
// pub fn get_token(iter: &mut std::iter::Peekable<std::str::Chars<'_>>, pos: &mut BytePos) -> Result<Option<Token>, Error> {
//     let mut buffer = String::new();
//     let lo = pos.clone();

//     let mut is_comment: bool = false;
//     let mut is_number: bool = true;

//     while let Some(c) = iter.next() {
//         if is_comment == true {
//             if c == '\n' || c == '\r' {
//                 // TODO :
//                 //we are removing the comments completely so that the parser does not become more complicated
//                 buffer.clear();
//                 is_comment = false;
//             //return Some(Token::new(TokenType::Comment, buffer));
//             } else {
//                 buffer.push(c);
//             }
//         } else {
//             match c {
//                 // TODO : Add '_' case? might be nice for naming variables but isn't in assignment.
//                 //Alpha characters
//                 'a'...'z' | 'A'...'Z' => {
//                     buffer.push(c);
//                     is_number = false;
//                     match iter.peek() {
//                         Some(' ') | Some('=') | Some('!') | Some('>') | Some('<') | Some('(')
//                         | Some(')') | Some('{') | Some('}') | Some('[') | Some(']') | Some(';')
//                         | Some('+') | Some('-') | Some('.') | Some('*') | Some('/') | Some(',')
//                         | Some('#') | Some('\r') | Some('\n') | None => {
//                             let token = check_keyword(&mut buffer, lo, *pos);
//                             *pos += 1;
//                             return Ok(Some(token));
//                         },
//                         Some(err) => {
//                             // Unexpected token, return error.
//                             return Err(Error::new());
//                         },
//                     }
//                 }

//                 //Numerics
//                 '0'...'9' => {
//                     buffer.push(c);
//                     match iter.peek() {
//                         Some(' ') | Some('+') | Some('-') | Some('/') | Some('*') | Some('=')
//                         | Some('!') | Some('>') | Some('<') | Some('{') | Some('[') | Some('(')
//                         | Some('}') | Some(']') | Some(')') | Some(';') | Some(',')
//                         | Some('\r') | Some('\n') | Some('\t') => {
//                             if is_number == true {
//                                 return Some(Token::new(TokenType::Number, buffer));
//                             } else if is_number == false {
//                                 return Some(Token::new(TokenType::Ident, buffer));
//                             }
//                         }
//                         _ => {}
//                     }
//                 }

//                 //Braces and Brackets
//                 '{' => {
//                     buffer.push(c);
//                     let token = build_token(TokenType::LeftBrace, buffer, lo, *pos);
//                     *pos += 1;
//                     return Ok(Some(token));
//                 }
//                 '[' => {
//                     buffer.push(c);
//                     let token = build_token(TokenType::LeftBracket, buffer, lo, *pos);
//                     *pos += 1;
//                     return Ok(Some(token));
//                     // return Some(Token::new(TokenType::LeftBracket, buffer));
//                 }
//                 '(' => {
//                     buffer.push(c);
//                     let token = build_token(TokenType::LeftPara, buffer, lo, *pos);
//                     *pos += 1;
//                     return Ok(Some(token));
//                     // return Some(Token::new(TokenType::LeftPara, buffer));
//                 }
//                 '}' => {
//                     buffer.push(c);
//                     let token = build_token(TokenType::RightBrace, buffer, lo, *pos);
//                     *pos += 1;
//                     return Ok(Some(token));
//                     // return Some(Token::new(TokenType::RightBrace, buffer));
//                 }
//                 ']' => {
//                     buffer.push(c);
//                     let token = build_token(TokenType::RightBracket, buffer, lo, *pos);
//                     *pos += 1;
//                     return Ok(Some(token));
//                     // return Some(Token::new(TokenType::RightBracket, buffer));
//                 }
//                 ')' => {
//                     buffer.push(c);
//                     let token = build_token(TokenType::RightPara, buffer, lo, *pos);
//                     *pos += 1;
//                     return Ok(Some(token));
//                     // return Some(Token::new(TokenType::RightPara, buffer));
//                 }

//                 //relOp characters will need an explicit peeknext
//                 '=' | '!' | '>' | '<' => {
//                     buffer.push(c);
//                     match iter.peek() {
//                         Some('=') | Some('!') | Some('>') | Some('<') => {}
//                         Some('-') => {
//                             if c == '<' {
//                             } else {
//                                 return Some(Token::new(TokenType::RelOp, buffer));
//                             }
//                         }
//                         _ => return Some(Token::new(TokenType::RelOp, buffer)),
//                     }
//                 }

//                 //Math Operators
//                 '+' => {
//                     buffer.push(c);
//                     return Some(Token::new(TokenType::AddOp, buffer));
//                 }
//                 '-' => {
//                     buffer.push(c);
//                     if buffer.as_str() == "<-" {
//                         return Some(Token::new(TokenType::AssignmentOp, buffer));
//                     } else {
//                         return Some(Token::new(TokenType::SubOp, buffer));
//                     }
//                 }
//                 '/' => {
//                     buffer.push(c);

//                     if *iter.peek().unwrap() == '/' {
//                         is_comment = true;
//                     } else {
//                         return Some(Token::new(TokenType::DivOp, buffer));
//                     }
//                 }
//                 '*' => {
//                     buffer.push(c);
//                     return Some(Token::new(TokenType::MulOp, buffer));
//                 }

//                 //Comment handlers
//                 '#' => {
//                     //Single comment token, take the rest of the line.
//                     buffer.push(c);
//                     is_comment = true;
//                 }
                

//                 //Comma Splitter
//                 ',' => {
//                     buffer.push(c);
//                     return Some(Token::new(TokenType::Comma, buffer));
//                 }

//                 //characters to ignore or remove (such as whitespace)
//                 ' ' => {}
//                 ';' => {
//                     buffer.push(c);
//                     return Some(Token::new(TokenType::SemiTermination, buffer));
//                 }
//                 '\'' => {}
//                 '\t' => {}
//                 '\r' => {}
//                 '\n' => {}

//                 //EOF and End of Main Function
//                 '.' => {
//                     buffer.push(c);
//                     return Some(Token::new(TokenType::ComputationEnd, buffer));
//                 }

//                 _ => {
//                     // Encountered some token that could not be lexed, return error
//                     *pos += 1;  // Add 1 to current position so that next token request starts at correct location.
//                     return Err(Error::new())
//                 }, //buffer.push(c),//for now we just build:
//             }
//         }

//         *pos += 1;
//     }

//     //leave in until dev done.
//     //very rudimentary method of breaking for loop of main function,
//     //there definitely has to be a better way!

//     // TODO : Find better method for finding EOF and remove this case.
//     //Some(Token::new(TokenType::Test, buffer))
//     None //If all above cases fall through, then we are at the end of our computation and thus we return None. This ends the Parsing.
// }

// /// When a string is completed, compare against this list of keywords.
// /// If a keyword is found, build token of type and return. Otherwise 
// /// it is a normal string or declaration, otherwise it is an Ident 
// /// token.
// fn check_keyword(key: &mut String, lo: BytePos, hi: BytePos) -> Token {
//     match key.as_str() {
//         "var" => build_token(TokenType::Var, key.to_string(), lo, hi),
//         "array" => build_token(TokenType::Array, key.to_string(), lo, hi),
//         "function" | "procedure" => build_token(TokenType::FuncDecl, key.to_string(), lo, hi),
//         "main" => build_token(TokenType::Computation, key.to_string(), lo, hi),
//         "let" => build_token(TokenType::Assignment, key.to_string(), lo, hi),
//         "call" => build_token(TokenType::FuncCall, key.to_string(), lo, hi),
//         "if" => build_token(TokenType::IfStatement, key.to_string(), lo, hi),
//         "then" => build_token(TokenType::ThenStatement, key.to_string(), lo, hi),
//         "else" => build_token(TokenType::ElseStatement, key.to_string(), lo, hi),
//         "fi" => build_token(TokenType::FiStatement, key.to_string(), lo, hi),
//         "while" => build_token(TokenType::WhileStatement, key.to_string(), lo, hi),
//         "do" => build_token(TokenType::DoStatement, key.to_string(), lo, hi),
//         "od" => build_token(TokenType::OdStatement, key.to_string(), lo, hi),
//         "return" => build_token(TokenType::ReturnStatement, key.to_string(), lo, hi),

//         ident => {
//             // Not one of the above keywords, it is therefore an ident, build 
//             // and return an Ident token.
//             build_token(TokenType::Ident, key.to_string(), lo, hi)
//         }
//     }
// }

// /// Super simple token builder function, takes necessary information and outputs a Token.
// /// Mostly using this to make span building easier and in a single location.
// fn build_token(token_ty: TokenType, buf: String, mut lo: BytePos, mut hi: BytePos) -> Token {
//     let span = Span::new(lo, hi);
//     Token::new(token_ty, buf, span)
// }
