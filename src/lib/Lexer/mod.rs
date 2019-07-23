pub mod token;

use std::iter::Peekable;
use std::str::Chars;

use self::token::Token;
use self::token::TokenType;
use lib::Utility::error::Error;
use lib::Utility::syntax_position::{BytePos, Span};

use std;

pub struct Lexer<'lctx,'lxr> {
    char_iter: &'lxr mut Chars<'lctx>,
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
        char_iter: &'lxr mut Chars<'lctx>
    ) -> Self {
        Lexer {
            char_iter,
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
        iter: &'lxr mut Chars<'lctx>
    ) -> Result<Vec<Token>, Vec<Error>> {
        let mut lexer = Lexer::new(iter);

    }

    fn check_keyword(
        &mut self,
    ) {
        match key.as_str() {
            "var" => build_token(TokenType::Var, key.to_string(), lo, hi),
            "array" => build_token(TokenType::Array, key.to_string(), lo, hi),
            "function" | "procedure" => build_token(TokenType::FuncDecl, key.to_string(), lo, hi),
            "main" => build_token(TokenType::Computation, key.to_string(), lo, hi),
            "let" => build_token(TokenType::Assignment, key.to_string(), lo, hi),
            "call" => build_token(TokenType::FuncCall, key.to_string(), lo, hi),
            "if" => build_token(TokenType::IfStatement, key.to_string(), lo, hi),
            "then" => build_token(TokenType::ThenStatement, key.to_string(), lo, hi),
            "else" => build_token(TokenType::ElseStatement, key.to_string(), lo, hi),
            "fi" => build_token(TokenType::FiStatement, key.to_string(), lo, hi),
            "while" => build_token(TokenType::WhileStatement, key.to_string(), lo, hi),
            "do" => build_token(TokenType::DoStatement, key.to_string(), lo, hi),
            "od" => build_token(TokenType::OdStatement, key.to_string(), lo, hi),
            "return" => build_token(TokenType::ReturnStatement, key.to_string(), lo, hi),

            ident => {
                // Not one of the above keywords, it is therefore an ident, build 
                // and return an Ident token.
                build_token(TokenType::Ident, key.to_string(), lo, hi)
            }
        }
    }

    /// Super simple token builder function, takes necessary information and outputs a Token.
    /// Mostly using this to make span building easier and in a single location.
    fn build_token(
        &mut self,
        token_ty: TokenType, 
    ) -> Token {
        let span = Span::new(lo, hi);
        Token::new(token_ty, buf, span)
    }

    fn collect_tokens(
        &mut self
    ) {
        let local_iter = self.char_iter.clone();
        local_iter.for_each(|c| {
            self.hi += 1;
            match c {
                // TODO : Add '_' case? might be nice for naming variables but isn't in assignment.
                //Alpha characters
                'a'...'z' | 'A'...'Z' => {
                    self.buffer.push(c);
                    is_number = false;
                    match iter.peek() {
                        Some(' ') | Some('=') | Some('!') | Some('>') | Some('<') | Some('(')
                        | Some(')') | Some('{') | Some('}') | Some('[') | Some(']') | Some(';')
                        | Some('+') | Some('-') | Some('.') | Some('*') | Some('/') | Some(',')
                        | Some('#') | Some('\r') | Some('\n') | None => {
                            let token = check_keyword(&mut buffer, lo, *pos);
                            *pos += 1;
                            return Ok(Some(token));
                        },
                        Some(err) => {
                            // Unexpected token, return error.
                            return Err(Error::new());
                        },
                    }
                }

                //Numerics
                '0'...'9' => {
                    buffer.push(c);
                    match iter.peek() {
                        Some(' ') | Some('+') | Some('-') | Some('/') | Some('*') | Some('=')
                        | Some('!') | Some('>') | Some('<') | Some('{') | Some('[') | Some('(')
                        | Some('}') | Some(']') | Some(')') | Some(';') | Some(',')
                        | Some('\r') | Some('\n') | Some('\t') => {
                            if is_number == true {
                                return Some(Token::new(TokenType::Number, buffer));
                            } else if is_number == false {
                                return Some(Token::new(TokenType::Ident, buffer));
                            }
                        }
                        _ => {}
                    }
                }

                //Braces and Brackets
                '{' => {
                    buffer.push(c);
                    let token = build_token(TokenType::LeftBrace, buffer, lo, *pos);
                    *pos += 1;
                    return Ok(Some(token));
                }
                '[' => {
                    buffer.push(c);
                    let token = build_token(TokenType::LeftBracket, buffer, lo, *pos);
                    *pos += 1;
                    return Ok(Some(token));
                    // return Some(Token::new(TokenType::LeftBracket, buffer));
                }
                '(' => {
                    buffer.push(c);
                    let token = build_token(TokenType::LeftPara, buffer, lo, *pos);
                    *pos += 1;
                    return Ok(Some(token));
                    // return Some(Token::new(TokenType::LeftPara, buffer));
                }
                '}' => {
                    buffer.push(c);
                    let token = build_token(TokenType::RightBrace, buffer, lo, *pos);
                    *pos += 1;
                    return Ok(Some(token));
                    // return Some(Token::new(TokenType::RightBrace, buffer));
                }
                ']' => {
                    buffer.push(c);
                    let token = build_token(TokenType::RightBracket, buffer, lo, *pos);
                    *pos += 1;
                    return Ok(Some(token));
                    // return Some(Token::new(TokenType::RightBracket, buffer));
                }
                ')' => {
                    buffer.push(c);
                    let token = build_token(TokenType::RightPara, buffer, lo, *pos);
                    *pos += 1;
                    return Ok(Some(token));
                    // return Some(Token::new(TokenType::RightPara, buffer));
                }

                //relOp characters will need an explicit peeknext
                '=' | '!' | '>' | '<' => {
                    buffer.push(c);
                    match iter.peek() {
                        Some('=') | Some('!') | Some('>') | Some('<') => {}
                        Some('-') => {
                            if c == '<' {
                            } else {
                                return Some(Token::new(TokenType::RelOp, buffer));
                            }
                        }
                        _ => return Some(Token::new(TokenType::RelOp, buffer)),
                    }
                }

                //Math Operators
                '+' => {
                    buffer.push(c);
                    return Some(Token::new(TokenType::AddOp, buffer));
                }
                '-' => {
                    buffer.push(c);
                    if buffer.as_str() == "<-" {
                        return Some(Token::new(TokenType::AssignmentOp, buffer));
                    } else {
                        return Some(Token::new(TokenType::SubOp, buffer));
                    }
                }
                '/' => {
                    buffer.push(c);

                    if *iter.peek().unwrap() == '/' {
                        is_comment = true;
                    } else {
                        return Some(Token::new(TokenType::DivOp, buffer));
                    }
                }
                '*' => {
                    buffer.push(c);
                    return Some(Token::new(TokenType::MulOp, buffer));
                }

                //Comment handlers
                '#' => {
                    //Single comment token, take the rest of the line.
                    buffer.push(c);
                    is_comment = true;
                }
                

                //Comma Splitter
                ',' => {
                    buffer.push(c);
                    return Some(Token::new(TokenType::Comma, buffer));
                }

                //characters to ignore or remove (such as whitespace)
                ' ' => {}
                ';' => {
                    buffer.push(c);
                    return Some(Token::new(TokenType::SemiTermination, buffer));
                }
                '\'' => {}
                '\t' => {}
                '\r' => {}
                '\n' => {}

                //EOF and End of Main Function
                '.' => {
                    buffer.push(c);
                    return Some(Token::new(TokenType::ComputationEnd, buffer));
                }

                _ => {
                    // Encountered some token that could not be lexed, return error
                    *pos += 1;  // Add 1 to current position so that next token request starts at correct location.
                    return Err(Error::new())
                }, //buffer.push(c),//for now we just build:
            }
        })
    }
}

pub fn tokenize(iter: &mut Peekable<Chars<'_>>) -> Result<Vec<Token>, Vec<Error>> {

}

//  As per Fabian's suggestion: use this function to request a token,
//then pass the token back. This function will take a string version of
//the code files, grab tokens, then throw back a single token when found.

// TODO : Potential optimization or reworks?

//This seems to work in separating tokens, but may need revisiting for better
//clarity of tokens or perhaps consolidation, we shall see.
pub fn get_token(iter: &mut std::iter::Peekable<std::str::Chars<'_>>, pos: &mut BytePos) -> Result<Option<Token>, Error> {
    let mut buffer = String::new();
    let lo = pos.clone();

    let mut is_comment: bool = false;
    let mut is_number: bool = true;

    while let Some(c) = iter.next() {
        if is_comment == true {
            if c == '\n' || c == '\r' {
                // TODO :
                //we are removing the comments completely so that the parser does not become more complicated
                buffer.clear();
                is_comment = false;
            //return Some(Token::new(TokenType::Comment, buffer));
            } else {
                buffer.push(c);
            }
        } else {
            match c {
                // TODO : Add '_' case? might be nice for naming variables but isn't in assignment.
                //Alpha characters
                'a'...'z' | 'A'...'Z' => {
                    buffer.push(c);
                    is_number = false;
                    match iter.peek() {
                        Some(' ') | Some('=') | Some('!') | Some('>') | Some('<') | Some('(')
                        | Some(')') | Some('{') | Some('}') | Some('[') | Some(']') | Some(';')
                        | Some('+') | Some('-') | Some('.') | Some('*') | Some('/') | Some(',')
                        | Some('#') | Some('\r') | Some('\n') | None => {
                            let token = check_keyword(&mut buffer, lo, *pos);
                            *pos += 1;
                            return Ok(Some(token));
                        },
                        Some(err) => {
                            // Unexpected token, return error.
                            return Err(Error::new());
                        },
                    }
                }

                //Numerics
                '0'...'9' => {
                    buffer.push(c);
                    match iter.peek() {
                        Some(' ') | Some('+') | Some('-') | Some('/') | Some('*') | Some('=')
                        | Some('!') | Some('>') | Some('<') | Some('{') | Some('[') | Some('(')
                        | Some('}') | Some(']') | Some(')') | Some(';') | Some(',')
                        | Some('\r') | Some('\n') | Some('\t') => {
                            if is_number == true {
                                return Some(Token::new(TokenType::Number, buffer));
                            } else if is_number == false {
                                return Some(Token::new(TokenType::Ident, buffer));
                            }
                        }
                        _ => {}
                    }
                }

                //Braces and Brackets
                '{' => {
                    buffer.push(c);
                    let token = build_token(TokenType::LeftBrace, buffer, lo, *pos);
                    *pos += 1;
                    return Ok(Some(token));
                }
                '[' => {
                    buffer.push(c);
                    let token = build_token(TokenType::LeftBracket, buffer, lo, *pos);
                    *pos += 1;
                    return Ok(Some(token));
                    // return Some(Token::new(TokenType::LeftBracket, buffer));
                }
                '(' => {
                    buffer.push(c);
                    let token = build_token(TokenType::LeftPara, buffer, lo, *pos);
                    *pos += 1;
                    return Ok(Some(token));
                    // return Some(Token::new(TokenType::LeftPara, buffer));
                }
                '}' => {
                    buffer.push(c);
                    let token = build_token(TokenType::RightBrace, buffer, lo, *pos);
                    *pos += 1;
                    return Ok(Some(token));
                    // return Some(Token::new(TokenType::RightBrace, buffer));
                }
                ']' => {
                    buffer.push(c);
                    let token = build_token(TokenType::RightBracket, buffer, lo, *pos);
                    *pos += 1;
                    return Ok(Some(token));
                    // return Some(Token::new(TokenType::RightBracket, buffer));
                }
                ')' => {
                    buffer.push(c);
                    let token = build_token(TokenType::RightPara, buffer, lo, *pos);
                    *pos += 1;
                    return Ok(Some(token));
                    // return Some(Token::new(TokenType::RightPara, buffer));
                }

                //relOp characters will need an explicit peeknext
                '=' | '!' | '>' | '<' => {
                    buffer.push(c);
                    match iter.peek() {
                        Some('=') | Some('!') | Some('>') | Some('<') => {}
                        Some('-') => {
                            if c == '<' {
                            } else {
                                return Some(Token::new(TokenType::RelOp, buffer));
                            }
                        }
                        _ => return Some(Token::new(TokenType::RelOp, buffer)),
                    }
                }

                //Math Operators
                '+' => {
                    buffer.push(c);
                    return Some(Token::new(TokenType::AddOp, buffer));
                }
                '-' => {
                    buffer.push(c);
                    if buffer.as_str() == "<-" {
                        return Some(Token::new(TokenType::AssignmentOp, buffer));
                    } else {
                        return Some(Token::new(TokenType::SubOp, buffer));
                    }
                }
                '/' => {
                    buffer.push(c);

                    if *iter.peek().unwrap() == '/' {
                        is_comment = true;
                    } else {
                        return Some(Token::new(TokenType::DivOp, buffer));
                    }
                }
                '*' => {
                    buffer.push(c);
                    return Some(Token::new(TokenType::MulOp, buffer));
                }

                //Comment handlers
                '#' => {
                    //Single comment token, take the rest of the line.
                    buffer.push(c);
                    is_comment = true;
                }
                

                //Comma Splitter
                ',' => {
                    buffer.push(c);
                    return Some(Token::new(TokenType::Comma, buffer));
                }

                //characters to ignore or remove (such as whitespace)
                ' ' => {}
                ';' => {
                    buffer.push(c);
                    return Some(Token::new(TokenType::SemiTermination, buffer));
                }
                '\'' => {}
                '\t' => {}
                '\r' => {}
                '\n' => {}

                //EOF and End of Main Function
                '.' => {
                    buffer.push(c);
                    return Some(Token::new(TokenType::ComputationEnd, buffer));
                }

                _ => {
                    // Encountered some token that could not be lexed, return error
                    *pos += 1;  // Add 1 to current position so that next token request starts at correct location.
                    return Err(Error::new())
                }, //buffer.push(c),//for now we just build:
            }
        }

        *pos += 1;
    }

    //leave in until dev done.
    //very rudimentary method of breaking for loop of main function,
    //there definitely has to be a better way!

    // TODO : Find better method for finding EOF and remove this case.
    //Some(Token::new(TokenType::Test, buffer))
    None //If all above cases fall through, then we are at the end of our computation and thus we return None. This ends the Parsing.
}

/// When a string is completed, compare against this list of keywords.
/// If a keyword is found, build token of type and return. Otherwise 
/// it is a normal string or declaration, otherwise it is an Ident 
/// token.
fn check_keyword(key: &mut String, lo: BytePos, hi: BytePos) -> Token {
    match key.as_str() {
        "var" => build_token(TokenType::Var, key.to_string(), lo, hi),
        "array" => build_token(TokenType::Array, key.to_string(), lo, hi),
        "function" | "procedure" => build_token(TokenType::FuncDecl, key.to_string(), lo, hi),
        "main" => build_token(TokenType::Computation, key.to_string(), lo, hi),
        "let" => build_token(TokenType::Assignment, key.to_string(), lo, hi),
        "call" => build_token(TokenType::FuncCall, key.to_string(), lo, hi),
        "if" => build_token(TokenType::IfStatement, key.to_string(), lo, hi),
        "then" => build_token(TokenType::ThenStatement, key.to_string(), lo, hi),
        "else" => build_token(TokenType::ElseStatement, key.to_string(), lo, hi),
        "fi" => build_token(TokenType::FiStatement, key.to_string(), lo, hi),
        "while" => build_token(TokenType::WhileStatement, key.to_string(), lo, hi),
        "do" => build_token(TokenType::DoStatement, key.to_string(), lo, hi),
        "od" => build_token(TokenType::OdStatement, key.to_string(), lo, hi),
        "return" => build_token(TokenType::ReturnStatement, key.to_string(), lo, hi),

        ident => {
            // Not one of the above keywords, it is therefore an ident, build 
            // and return an Ident token.
            build_token(TokenType::Ident, key.to_string(), lo, hi)
        }
    }
}

/// Super simple token builder function, takes necessary information and outputs a Token.
/// Mostly using this to make span building easier and in a single location.
fn build_token(token_ty: TokenType, buf: String, mut lo: BytePos, mut hi: BytePos) -> Token {
    let span = Span::new(lo, hi);
    Token::new(token_ty, buf, span)
}
