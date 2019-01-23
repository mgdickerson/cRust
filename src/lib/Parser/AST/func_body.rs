use lib::Lexer::token::TokenCollection;
use lib::Lexer::token::TokenType;
use Parser::AST::assignment::Assignment;
use Parser::AST::if_stmt::IfStmt;
use Parser::AST::while_stmt::WhileStmt;
use Parser::AST::func_call::FuncCall;
use Parser::AST::return_stmt::ReturnStmt;

use super::{Node, NodeId, NodeData, IRManager, Value, ValTy, Op, InstTy};
use super::Graph;
use lib::Graph::graph_manager::GraphManager;

#[derive(Debug,Clone)]
enum Stmt {
    assignment(Assignment),
    if_stmt(IfStmt),
    while_stmt(WhileStmt),
    funcCall(FuncCall),
    return_stmt(ReturnStmt),
}

#[derive(Debug,Clone)]
pub struct FuncBody {
    node_type: TokenType,
    stmt_vec: Vec<Stmt>,
}

impl FuncBody {
    pub fn new(tc: &mut TokenCollection) -> Self {
        let mut stmt_vec = Vec::new();

        while let Some(next_token) = tc.peek_next_token_type() {
            match next_token {
                TokenType::Assignment => {
                    stmt_vec.push(Stmt::assignment(Assignment::new(tc)));
                },
                TokenType::IfStatement => {
                    stmt_vec.push(Stmt::if_stmt(IfStmt::new(tc)));
                },
                TokenType::WhileStatement => {
                    stmt_vec.push(Stmt::while_stmt(WhileStmt::new(tc)));
                },
                TokenType::FuncCall => {
                    stmt_vec.push(Stmt::funcCall(FuncCall::new(tc)));

                    match tc.peek_next_token_type() {
                        Some(TokenType::SemiTermination) => {
                            //consume then resume cycle
                            tc.get_next_token();
                            continue;
                        },
                        Some(TokenType::RightBrace) | Some(TokenType::FiStatement) |
                        Some(TokenType::OdStatement) | Some(TokenType::ElseStatement) => {
                            //fall through
                            continue
                        },
                        None => {
                            // Compiler Error :
                            panic!("Expected some form of termination after function call in function body.");
                        },
                        err => {
                            // Compiler Error :
                            panic!("Expected termination sequence after FuncCall, found unexpected Token: {:?}");
                        },
                    }
                },
                TokenType::ReturnStatement => {
                    stmt_vec.push(Stmt::return_stmt(ReturnStmt::new(tc)));
                },

                //end of function body sequences
                TokenType::RightBrace | TokenType::FiStatement |
                TokenType::OdStatement | TokenType::ElseStatement => {
                    //consume token? or just return?
                    break
                },

                // Compiler Error :
                err => {
                    panic!("Unable to parse {:?} within function body.", err);
                }
            }
        }

        FuncBody { node_type: TokenType::FuncBody, stmt_vec }
    }

    pub fn get_value(&self) -> Vec<Stmt>  {
        return self.stmt_vec.to_vec()
    }

    pub fn get_type(&self) -> TokenType {
        self.node_type.clone()
    }

    pub fn to_ir(self, graph_manager: &mut GraphManager, irm: &mut IRManager) {
        for stmt in self.stmt_vec {
            match stmt {
                Stmt::assignment(assign) => {
                    assign.to_ir(graph_manager,irm);
                },
                Stmt::if_stmt(if_st) => {
                    if_st.to_ir(graph_manager,irm);
                },
                Stmt::while_stmt(wh_st) => {

                },
                Stmt::funcCall(fn_cl) => {

                },
                Stmt::return_stmt(rt) => {

                },
            }
        }
    }

}