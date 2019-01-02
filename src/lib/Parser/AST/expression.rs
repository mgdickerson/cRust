use lib::Lexer::token::Token;
use lib::Lexer::token::TokenType;
use lib::Lexer::token::TokenCollection;
use Parser::AST::term::Term;

#[derive(Debug,Clone)]
pub struct Expression {
    node_type: TokenType,
    terms: Vec<Term>,
    math_ops: Vec<Token>,
}

impl Expression {
    pub fn new(tc: &mut TokenCollection) -> Self {
        let mut terms : Vec<Term> = vec!();
        let mut operations : Vec<Token> = vec!();
        terms.push(Term::new(tc));

        loop {
            //handle MathOp possibility
            match tc.peek_next_token_type() {
                Some(TokenType::MathOp) => {
                    //MathOp found, call another term.
                    operations.push(tc.get_next_token().unwrap());    // consume the MathOp
                    terms.push(Term::new(tc));
                },
                None => {
                    // Compiler Error :
                    panic!("Unexpected EOF in expression.");
                },
                _ => {
                    //If there is no MathOp, return. Dont do any other debugging or logic here.
                    return Expression { node_type: TokenType::Expression, terms, math_ops: operations }
                },
            }
        }
    }

    pub fn get_value(&self) -> (Vec<Term>, Vec<Token>)  {
        return (self.terms.to_vec(), self.math_ops.to_vec())
    }

    pub fn get_type(&self) -> TokenType {
        self.node_type.clone()
    }
}