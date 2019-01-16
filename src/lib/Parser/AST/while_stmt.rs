use lib::Lexer::token::TokenCollection;
use lib::Lexer::token::TokenType;
use Parser::AST::relation::Relation;
use Parser::AST::func_body::FuncBody;

use super::{Node, NodeId, NodeData, IRManager, Value, ValTy, Op, InstTy};
use super::Graph;

#[derive(Debug,Clone)]
pub struct WhileStmt {
    node_type: TokenType,
    relation: Relation,
    body: FuncBody,
}

impl WhileStmt {
    pub fn new(tc: &mut TokenCollection) -> Self {
        let mut relation;
        let mut body;

        match tc.get_next_token().expect("While Statement Error").get_type() {
            TokenType::WhileStatement => {
                //expected token was found, next do relation
                relation = Relation::new(tc);
            },
            err => {
                // Compiler Error :
                panic!("Expected While statement, found unexpected Token: {:?}", err);
            },
        }

        match tc.peek_next_token_type() {
            Some(TokenType::DoStatement) => {
                tc.get_next_token();
                body = FuncBody::new(tc);
            },
            None => {
                // Compiler Error :
                panic!("Unexpected End of File, expected do statement.");
            },
            err => {
                // Compiler Error :
                panic!("Expected do statement, found unexpected Token: {:?}", err);
            },
        }

        match tc.peek_next_token_type() {
            Some(TokenType::OdStatement) => {
                tc.get_next_token();
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
                        panic!("Expected to find ';' or end of block after Od statement, found unexpected Token: {:?}", err);
                    },
                }
            },
            None => {
                // Compiler Error :
                panic!("Unexpected End of File, expected Od Token.");
            },
            err => {
                // Compiler Error :
                panic!("Expected Od Token, found unexpected Token: {:?}", err);
            },
        }

        WhileStmt { node_type: TokenType::WhileStatement, relation, body }
    }

    pub fn get_value(&self) -> (Relation, FuncBody)  {
        return (self.relation.clone(), self.body.clone())
    }

    pub fn get_type(&self) -> TokenType {
        self.node_type.clone()
    }
}