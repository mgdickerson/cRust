use lib::Lexer::token::TokenCollection;
use lib::Lexer::token::TokenType;
use Parser::AST::ident::Ident;
use Parser::AST::expression::Expression;

pub struct Designator {
    ident: Ident,
    expressions: Vec<Expression>,
}

impl Designator {
    pub fn new(tc: &mut TokenCollection) -> Self {
        match tc.peek_next_token_type() {
            Some(TokenType::Ident) => {
                let current_ident = Ident::new(tc.get_next_token().unwrap()));

                while let Some(next_token) = tc.peek_next_token_type() {
                    match next_token {
                        TokenType::LeftBrace => {
                            //consume left brace
                            tc.get_next_token();

                            expression(tc);

                            //consume next token if right brace
                            match tc.peek_next_token_type() {
                                Some(TokenType::RightBrace) => {
                                    //consume right brace
                                    tc.get_next_token();
                                },
                                None => {
                                    // Compiler Error :
                                    panic!("Unexpected EOF, expected ']' token for designator.");
                                },
                                err => {
                                    // Compiler Error :
                                    panic!("Unexpected Token: {:?}, expected ']' token for designator.", err);
                                },
                            }
                        },
                        _ => {
                            //ident already collected, bail. no need for error handling here.
                            Designator{ ident : current_ident,
                                expressions : current_expr }
                        },
                    }
                }
            }

            None => {
                // Compiler Error :
                panic!("Unexpected EOF, expected Ident token for designator.");
            },
            err => {
                // Compiler Error :
                panic!("Expected Ident Token in designator, found unexpected Token: {:?}", err);
            },
        }
        Designator{ ident : Ident::new(tc.unwrap()), expressions : vec![] }
    }
}