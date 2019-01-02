use lib::Lexer::token::TokenCollection;
use lib::Lexer::token::TokenType;
use lib::Lexer::token::Token;
use Parser::AST::factor::Factor;

#[derive(Debug,Clone)]
pub struct Term {
    node_type: TokenType,
    factors: Vec<Factor>,
    operations: Vec<Token>,
}

impl Term {
    pub fn new(tc: &mut TokenCollection) -> Self {
        let mut factors : Vec<Factor> = vec!();
        let mut operations : Vec<Token> = vec!();
        factors.push(Factor::new(tc));

        loop {
            //handle MulOp possibility
            match tc.peek_next_token_type() {
                Some(TokenType::MulOp) => {
                    //MulOp found, consume then call factor again
                    operations.push(tc.get_next_token().unwrap());
                    factors.push(Factor::new(tc));
                },
                None => {
                    // Compiler Error :
                    panic!("Unexpected EOF in term.");
                },
                _ => {
                    //If no MulOp, return
                    return Term { node_type: TokenType::Term, factors, operations }
                },
            }
        }
    }

    pub fn get_value(&self) -> (Vec<Factor>, Vec<Token>)  {
        return (self.factors.to_vec(), self.operations.to_vec())
    }

    pub fn get_type(&self) -> TokenType {
        self.node_type.clone()
    }
}