pub mod token;

use self::token::Token;
use self::token::TokenType;

use std;

//  As per Fabian's suggestion: use this function to request a token,
//then pass the token back. This function will take a string version of
//the code files, grab tokens, then throw back a single token when found.

// TODO : Potential optimization or reworks?

//This seems to work in separating tokens, but may need revisiting for better
//clarity of tokens or perhaps consolidation, we shall see.
pub fn get_token(iter: &mut std::iter::Peekable<std::str::Chars<'_>>) -> Option<Token> {
    let mut buffer = String::new();

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
            }
            else { 
                buffer.push(c);
            }
        }
        else {
            match c {
                // TODO : Add '_' case? might be nice for naming variables but isn't in assignment.
                //Alpha characters
                'a' ... 'z' | 'A' ... 'Z' => {
                    buffer.push(c);
                    is_number = false;
                    match iter.peek() {
                        Some(' ') | Some('=') | Some('!') | Some('>') |
                        Some('<') | Some('(') | Some(')') | Some('{') |
                        Some('}') | Some('[') | Some(']') | Some(';') |
                        Some('+') | Some('-') | Some('.') | Some('*') |
                        Some('/') | Some(',') | Some('#') | Some('\r')|
                        Some('\n') => {
                            

                            match check_keyword(&mut buffer) {
                                Some(x) => return Some(x),
                                None => { return Some(Token::new(TokenType::Ident, buffer)) },
                            }
                        },
                        None => { return Some(Token::new(TokenType::Ident, buffer)) },
                        Some(_) => {},
                    }
                },

                //Numerics
                '0' ... '9' => {
                    buffer.push(c);
                    match iter.peek() {
                        Some(' ') | Some('+') | Some('-') | Some('/') |
                        Some('*') | Some('=') | Some('!') | Some('>') |
                        Some('<') | Some('{') | Some('[') | Some('(') |
                        Some('}') | Some(']') | Some(')') | Some(';') |
                        Some(',') | Some('\r')| Some('\n')| Some('\t') => {
                            if is_number == true {
                                return Some(Token::new(TokenType::Number, buffer))
                            }
                            else if is_number == false {
                                return Some(Token::new(TokenType::Ident, buffer))
                            }
                        },
                        _ => {},
                    }
                },

                //Braces and Brackets
                '{' => { buffer.push(c); return Some(Token::new(TokenType::LeftBrace, buffer)) },
                '[' => { buffer.push(c); return Some(Token::new(TokenType::LeftBracket, buffer)) },
                '(' => { buffer.push(c); return Some(Token::new(TokenType::LeftPara, buffer)) },
                '}' => { buffer.push(c); return Some(Token::new(TokenType::RightBrace, buffer)) },
                ']' => { buffer.push(c); return Some(Token::new(TokenType::RightBracket, buffer)) },
                ')' => { buffer.push(c); return Some(Token::new(TokenType::RightPara, buffer)) },

                //relOp characters will need an explicit peeknext
                '=' | '!' | '>' | '<' => {
                    buffer.push(c);
                    match iter.peek() {
                        Some('=') | Some('!') | Some('>') | Some('<') => {},
                        Some ('-') => {
                            if c == '<' {}
                            else { return Some(Token::new(TokenType::RelOp, buffer)) }
                        },
                        _ => { return Some(Token::new(TokenType::RelOp, buffer)) },
                    }
                },

                //Math Operators
                '+' => { buffer.push(c); return Some(Token::new(TokenType::AddOp, buffer)) },
                '-' => {
                    buffer.push(c);
                    if buffer.as_str() == "<-" {
                        return Some(Token::new(TokenType::AssignmentOp, buffer))
                    }
                    else {
                        return Some(Token::new(TokenType::SubOp, buffer))
                    }
                },

                //Comment handlers
                '#' => {
                    //Single comment token, take the rest of the line.
                    buffer.push(c);
                    is_comment = true;
                },
                '/' => {
                    buffer.push(c);

                    if *iter.peek().unwrap() == '/' {
                        is_comment = true;
                    }
                    else { return Some(Token::new(TokenType::DivOp, buffer)) }
                },
                '*' => { buffer.push(c); return Some(Token::new(TokenType::MulOp, buffer)) },

                //Comma Splitter
                ',' => { buffer.push(c); return Some(Token::new(TokenType::Comma, buffer)) },

                //characters to ignore or remove (such as whitespace)
                ' ' => {},
                ';' => { buffer.push(c); return Some(Token::new(TokenType::SemiTermination, buffer)) },
                '\'' => {},
                '\t' => {},
                '\r' => {},
                '\n' => {},
                
                //EOF and End of Main Function
                '.' => { buffer.push(c); return Some(Token::new(TokenType::ComputationEnd, buffer)) },

                _ => panic!("Should never reach this case! Character that gave error: {}", c), //buffer.push(c),//for now we just build:  
            }
        }
    }

    //leave in until dev done.
    //very rudimentary method of breaking for loop of main function,
    //there definitely has to be a better way!

    // TODO : Find better method for finding EOF and remove this case.
    //Some(Token::new(TokenType::Test, buffer))
    None    //If all above cases fall through, then we are at the end of our computation and thus we return None. This ends the Parsing.
}

fn check_keyword(key: &mut String) -> Option<Token> {
    match key.as_str() {
        "var" => { Some(Token::new(TokenType::Var, key.to_string())) },
        "array" => { Some(Token::new(TokenType::Array, key.to_string())) },
        "function" | "procedure" => { Some(Token::new(TokenType::FuncDecl, key.to_string())) },
        "main" => { Some(Token::new(TokenType::Computation, key.to_string())) },
        "let" => { Some(Token::new(TokenType::Assignment, key.to_string())) },
        "call" => { Some(Token::new(TokenType::FuncCall, key.to_string())) },
        "if" => { Some(Token::new(TokenType::IfStatement, key.to_string())) },
        "then" => { Some(Token::new(TokenType::ThenStatement, key.to_string())) },
        "else" => { Some(Token::new(TokenType::ElseStatement, key.to_string())) },
        "fi" => { Some(Token::new(TokenType::FiStatement, key.to_string())) },
        "while" => { Some(Token::new(TokenType::WhileStatement, key.to_string())) },
        "do" => { Some(Token::new(TokenType::DoStatement, key.to_string())) },
        "od" => { Some(Token::new(TokenType::OdStatement, key.to_string())) },
        "return" => { Some(Token::new(TokenType::ReturnStatement, key.to_string())) },
        
        //If no keyword is found, it returns None, and the string must be an ident of some sort.
        _ => None,
    }
}