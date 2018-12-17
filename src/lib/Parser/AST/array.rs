use lib::Lexer::token::TokenCollection;
use lib::Lexer::token::TokenType;
use Parser::AST;

pub struct Array {
    arrayDepthVec: Vec<AST::number::Number>,
    identList: Vec<AST::ident::Ident>,
    debugLine: String,
}

impl Array {
    pub fn new(tc: &mut TokenCollection) -> Self {
        let mut varList : Vec<AST::ident::Ident> = vec!();
        let mut numList : Vec<AST::number::Number> = vec!();

        match tc.get_next_token().expect("Array Error").get_type() {
            TokenType::Array => {
                // proper action, all is well.
            },
            err => {
                // Compiler Error : 
                panic!("Expected Array declaration, found unexpected Token: {:?}", err);
            },
        }

        match tc.peek_next_token_type() {
            Some(TokenType::LeftBrace) => {
                //all is well, proceed through
            },
            None => {
                // Compiler Error : 
                panic!("Expected Array '[' Token, found EOF.");
            },
            err => {
                // Compiler Error : 
                panic!("Expected '[' Token in array declaration, found unexpected Token: {:?}", err);
            },
        }

        while let Some(next_token) = tc.peek_next_token_type() {
            match next_token {
                TokenType::LeftBrace => {
                    //should this update depth of array? Or do these just get consumed?
                    //for now, just consume. 
                    // TODO : Confirm it is the right bracket type. '['
                    tc.get_next_token();

                    match tc.peek_next_token_type() {
                        Some(TokenType::Number) => {
                            numList.push(AST::number::Number::new(tc.get_next_token().unwrap()));
                        },
                        None => {
                            // Compiler Error : 
                            panic!("Expected Array index Number, found EOF.");
                        },
                        err => {
                            // Compiler Error : 
                            panic!("Expected Number Token in array index, found unexpected Token: {:?}", err);
                        },
                    }

                    match tc.peek_next_token_type() {
                        Some(TokenType::RightBrace) => {
                            tc.get_next_token();
                        },
                        None => {
                            // Compiler Error : 
                            panic!("Expected Array ']', found EOF.");
                        },
                        err => {
                            // Compiler Error : 
                            panic!("Expected ']' Token in array declaration, found unexpected Token: {:?}", err);
                        },
                    }
                },
                TokenType::Ident => {
                    // first ident case found, break from loop. 
                    break;
                }
                err => {
                    // Compiler Error : 
                    panic!("Expected Array T: {:?}", err);
                },
            }
        }

        while let Some(next_token) = tc.peek_next_token_type() {
            match next_token {
                TokenType::Ident => {
                    varList.push(AST::ident::Ident::new(tc.get_next_token().unwrap()));
                },
                TokenType::Comma => {
                    tc.get_next_token();
                    continue;
                }
                TokenType::SemiTermination => {
                    tc.get_next_token();
                    break;
                }
                err => {
                    // Compiler Error : 
                    panic!("Expected Ident Token in array declaration, found unexpected Token: {:?}", err);
                },
            }
        }

        Array { arrayDepthVec: numList,
            identList: varList,
            debugLine: String::from("test") }
    }
}