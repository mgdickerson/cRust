use lib::Lexer::token::TokenCollection;
use lib::Lexer::token::TokenType;
use lib::Lexer::token::Token;
use Parser::AST::expression::Expression;

#[derive(Debug,Clone)]
pub struct Relation {
    node_type: TokenType,
    leftExp: Expression,
    relOp: Token,
    rightExp: Expression,
}

impl Relation {
    pub fn new(tc: &mut TokenCollection) -> Self {
        let leftExp = Expression::new(tc);
        let mut relOp = Token::new(TokenType::None, String::from(""));

        match tc.peek_next_token_type() {
            Some(TokenType::RelOp) => {
                //consume token
                relOp = tc.get_next_token().unwrap();
            },
            None => {
                // Compiler Error :
                panic!("Expected RelOp token, found EOF.");
            },
            err => {
                // Compiler Error :
                panic!("Expected RelOp token, unexpected Token {:?} was found instead.", err);
            },
        }

        let rightExp = Expression::new(tc);

        //relation is built, return
        Relation{ node_type: TokenType::Relation, leftExp, relOp, rightExp }
    }

    pub fn get_value(&self) -> (Expression, Token, Expression)  {
        return (self.leftExp.clone(), self.relOp.clone(), self.rightExp.clone())
    }

    pub fn get_type(&self) -> TokenType {
        self.node_type.clone()
    }
}