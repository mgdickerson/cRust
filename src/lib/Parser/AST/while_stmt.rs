use lib::Lexer::token::TokenCollection;
use lib::Lexer::token::TokenType;
use Parser::AST::relation::Relation;
use Parser::AST::func_body::FuncBody;

use super::{Node, NodeId, NodeType, NodeData, IRManager, Value, ValTy, Op, InstTy};
use super::Graph;
use lib::Graph::graph_manager::GraphManager;

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

    pub fn get_value(&self) -> (Relation, FuncBody) {
        return (self.relation.clone(), self.body.clone())
    }

    pub fn get_type(&self) -> TokenType {
        self.node_type.clone()
    }

    pub fn to_ir(self, graph_manager: &mut GraphManager, irm: &mut IRManager) {
        /// General Order:
        /// - enters on "Main Node"
        /// - generate loop-node
        /// - connect loop head to main-node
        /// - go through loop-body, generate loop-bottom
        /// - connect loop-bottom to main-node
        /// - generate phi-node
        /// - connect main-node to phi
        /// - phi node is new "Main Node"

        let main_node = graph_manager.clone_node_index();
        // Handy for return instruction later
        let return_point = self.relation.to_ir(graph_manager,irm, Value::new(ValTy::con(-1)));

        // Generate loop-head
        graph_manager.new_node(irm, NodeType::while_node);
        let loop_node_top = graph_manager.clone_node_index();
        // Connect main_node to loop_node_top
        graph_manager.add_edge(main_node,loop_node_top);

        // Go through loop body
        self.body.to_ir(graph_manager,irm);
        // Add return branch instruction to "new main node"
        let bra_return = irm.build_op_y(return_point,InstTy::bra);
        graph_manager.add_instruction(bra_return);

        // Generate new loop bottom node
        let loop_node_bottom = graph_manager.clone_node_index();

        // Generate phi node
        graph_manager.new_node(irm, NodeType::phi_node);
        let phi_node = graph_manager.clone_node_index();

        graph_manager.add_edge(main_node,phi_node);
        graph_manager.add_edge(loop_node_bottom,main_node);
    }
}