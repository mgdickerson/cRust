use lib::Lexer::token::Token;
use lib::Lexer::token::TokenType;
use lib::Lexer::token::TokenCollection;

#[derive(Debug,Clone)]
pub struct Expression {
    terms: Vec<Terms>,
    math_ops: Vec<Token>,
}

impl Expression {
    pub fn new(tc: &mut TokenCollection) -> Self {
        term(tc);

        loop {
            //handle MathOp possibility
            match tc.peek_next_token_type() {
                Some(TokenType::MathOp) => {
                    //MathOp found, call another term.
                    tc.get_next_token();    // consume the MathOp
                    term(tc);
                },
                None => {
                    // Compiler Error :
                    panic!("Unexpected EOF in expression.");
                },
                _ => {
                    //If there is no MathOp, return. Dont do any other debugging or logic here.
                    return
                },
            }
        }
    }
}