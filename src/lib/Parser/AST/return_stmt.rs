use lib::Lexer::token::TokenCollection;
use lib::Lexer::token::TokenType;
use Parser::AST::expression::Expression;

use super::{Node, NodeId, NodeData, IRGraphManager, Value, ValTy, Op, InstTy};
use super::Graph;

#[derive(Debug,Clone)]
pub struct ReturnStmt {
    node_type: TokenType,
    expression: Expression,
}

impl ReturnStmt {
    pub fn new(tc: &mut TokenCollection) -> Self {
        match tc.get_next_token().expect("Return Statement Error").get_type() {
            TokenType::ReturnStatement => {
                // return token found, pass through to handle expression.
                // Otherwise, error handle.
            },
            // TODO : fix up to proper error handler
            err => { println!("Expected Return Statement, found unexpected Token: {:?}", err); },  //proper method of error handling unexpected tokens
        }

        let expression = Expression::new(tc);

        match tc.peek_next_token_type() {
            Some(TokenType::SemiTermination) => {
                //Found ';' so there are likely to be more statements. Consume and return.
                tc.get_next_token();
            },
            // All Possible Ending Sequences where ';' may not be necessary.
            Some(TokenType::FiStatement) | Some(TokenType::OdStatement) |
            Some(TokenType::RightBrace) | Some(TokenType::ElseStatement) => {
                //';' not required, return without consuming token.
            },
            None => {
                // Compiler Error :
                panic!("End of file found, do should be appended by '}' if end of statement");
            },
            err => {
                // Compiler Error :
                panic!("Expected to find ';' or end  statement, found unexpected Token: {:?}", err);
            },
        }

        ReturnStmt { node_type: TokenType::ReturnStatement, expression }
    }

    pub fn get_value(&self) -> Expression  {
        return self.expression.clone()
    }

    pub fn get_type(&self) -> TokenType {
        self.node_type.clone()
    }

    pub fn to_ir(self, irgm: &mut IRGraphManager) {
        let ret_val = self.expression.to_ir(irgm);

        // This will be a special instruction that always returns values on register R27;
        let ret_inst = irgm.build_op_x(ret_val.expect("return calls should always return an expr"), InstTy::ret);

    }

    pub fn scan_globals(&self, irgm : &mut IRGraphManager) {
        irgm.variable_manager().active_function().set_return(true);
        self.expression.scan_globals(irgm);
    }
}