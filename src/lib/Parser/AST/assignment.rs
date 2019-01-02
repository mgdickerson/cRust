use lib::Lexer::token::TokenCollection;
use lib::Lexer::token::TokenType;
use Parser::AST::designator::Designator;
use Parser::AST::expression::Expression;

#[derive(Debug,Clone)]
pub struct Assignment {
    node_type: TokenType,
    designator: Designator,
    expression: Expression,
}

impl Assignment {
    pub fn new(tc: &mut TokenCollection) -> Self {
        let mut designator;
        let mut expression;

        match tc.get_next_token().expect("Assignment Error").get_type() {
            TokenType::Assignment => {
                //expected assignment token found.
            },
            err => {
                // Compiler Error :
                panic!("Expected to find Assignment token, found unexpected Token: {:?}", err);
            },
        }

        match tc.peek_next_token_type() {
            Some(TokenType::Ident) => {
                designator = Designator::new(tc);
            },
            err => {
                // Compiler Error :
                panic!("Expected Designator for assignment variable, found unexpected Token: {:?}", err);
            },
        }

        match tc.get_next_token().expect("Assignment Op Error").get_type() {
            TokenType::AssignmentOp => {
                //expected assignment operator found, proceed to expression.
                expression = Expression::new(tc);
            },
            err => {
                // Compiler Error :
                panic!("Expected Assignment Operator '<-', found unexpected Token: {:?}", err);
            },
        }

        match tc.peek_next_token_type() {
            Some(TokenType::SemiTermination) => {
                //consume token, return.
                tc.get_next_token();
            },
            // All Possible Ending Sequences where ';' may not be necessary.
            Some(TokenType::FiStatement) | Some(TokenType::OdStatement) |
            Some(TokenType::RightBrace) | Some(TokenType::ElseStatement) => {
                //';' not required, return without consuming token.
            },
            None => {
                // Compiler Error :
                panic!("Expected end of assignment, found EOF.");
            },
            err => {
                // Compiler Error :
                panic!("Expected end of assignment, found unexpected Token: {:?}", err);
            },
        }

        Assignment { node_type: TokenType::Assignment, designator, expression }
    }

    pub fn get_value(&self) -> (Designator, Expression)  {
        return (self.designator.clone(), self.expression.clone())
    }

    pub fn get_type(&self) -> TokenType {
        self.node_type.clone()
    }
}