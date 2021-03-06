use lib::Lexer::token::TokenCollection;
use lib::Lexer::token::TokenType;
use Parser::AST::expression::Expression;
use Parser::AST::ident::Ident;

use super::IRGraphManager;

#[derive(Debug, Clone)]
pub struct Designator {
    node_type: TokenType,
    ident: Ident,
    expressions: Vec<Expression>,
}

impl Designator {
    pub fn new(tc: &mut TokenCollection) -> Self {
        let mut expList: Vec<Expression> = vec![];
        let mut tokenType = TokenType::None;

        match tc.peek_next_token_type() {
            Some(TokenType::Ident) => {
                let current_ident = Ident::new(tc);
                tokenType = TokenType::Designator;

                while let Some(next_token) = tc.peek_next_token_type() {
                    match next_token {
                        TokenType::LeftBracket => {
                            // consume left brace
                            tc.get_next_token();

                            expList.push(Expression::new(tc));

                            // consume next token if right brace
                            match tc.peek_next_token_type() {
                                Some(TokenType::RightBracket) => {
                                    // consume right brace
                                    tc.get_next_token();
                                }
                                None => {
                                    // Compiler Error :
                                    panic!("Unexpected EOF, expected ']' token for designator.");
                                }
                                err => {
                                    // Compiler Error :
                                    panic!("Unexpected Token: {:?}, expected ']' token for designator.", err);
                                }
                            }
                        }
                        _ => {
                            // ident already collected, bail. no need for error handling here.
                            return Designator {
                                node_type: tokenType,
                                ident: current_ident,
                                expressions: expList,
                            };
                        }
                    }
                }
            }

            None => {
                // Compiler Error :
                panic!("Unexpected EOF, expected Ident token for designator.");
            }
            err => {
                // Compiler Error :
                panic!(
                    "Expected Ident Token in designator, found unexpected Token: {:?}",
                    err
                );
            }
        }

        // Compiler Error : Should not reach this stage.
        Designator {
            node_type: tokenType,
            ident: Ident::new(tc),
            expressions: vec![],
        }
    }

    pub fn get_value(&self) -> (Ident, Vec<Expression>) {
        return (self.ident.clone(), self.expressions.to_vec());
    }

    pub fn get_type(&self) -> TokenType {
        self.node_type.clone()
    }

    pub fn scan_globals(&self, irgm: &mut IRGraphManager) {
        let (ident, expr_array) = self.get_value();

        if expr_array.is_empty() {
            if irgm.variable_manager().is_global(&ident.get_value()) {
                if !irgm
                    .variable_manager()
                    .active_function()
                    .load_param_list()
                    .contains(&ident.get_value())
                {
                    irgm.variable_manager()
                        .active_function()
                        .add_global(&ident.get_value());
                } else {
                    //println!("Global var {} is being overwritten by local param.", ident.get_value());
                }
            }
        }
    }
}
