use lib::Lexer::token::Token;
use lib::Lexer::token::TokenType;
use lib::Lexer::token::TokenCollection;
use Parser::AST::term::Term;

#[derive(Debug,Clone)]
pub enum ExpList {
    term(Term),
    math_op(Token),
}

#[derive(Debug,Clone)]
pub struct Expression {
    node_type: TokenType,
    exp_list: Vec<ExpList>,
}

impl Expression {
    pub fn new(tc: &mut TokenCollection) -> Self {
        let mut exp_list = Vec::new();

        exp_list.push(ExpList::term(Term::new(tc)));

        loop {
            //handle MathOp possibility
            match tc.peek_next_token_type() {
                Some(TokenType::AddOp) | Some(TokenType::SubOp) => {
                    //MathOp found, call another term.

                    exp_list.push(ExpList::math_op(tc.get_next_token().unwrap()));
                    exp_list.push(ExpList::term(Term::new(tc)));
                },
                None => {
                    // Compiler Error :
                    panic!("Unexpected EOF in expression.");
                },
                _ => {
                    //If there is no MathOp, return. Dont do any other debugging or logic here.
                    return Expression { node_type: TokenType::Expression, exp_list }
                },
            }
        }
    }

    pub fn get_value(&self) -> Vec<ExpList>  {
        return self.exp_list.clone()
    }

    pub fn get_type(&self) -> TokenType {
        self.node_type.clone()
    }
}