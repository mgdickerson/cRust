use lib::Lexer::token::TokenCollection;
use lib::Lexer::token::TokenType;
use Parser::AST::relation::Relation;
use Parser::AST::func_body::FuncBody;

use super::{Node, NodeId, NodeData, IRManager, Value, ValTy, Op, InstTy};
use super::Graph;
use lib::Graph::graph_manager::GraphManager;

#[derive(Debug,Clone)]
pub struct IfStmt {
    node_type: TokenType,
    relation: Relation,
    funcIfBody: FuncBody,
    funcElseBody: Option<FuncBody>,
}

impl IfStmt {
    pub fn new(tc: &mut TokenCollection) -> Self {
        let mut relation;
        let mut funcIfBody;
        let mut funcElseBody = Option::None;

        match tc.get_next_token().expect("If Statment Error").get_type() {
            TokenType::IfStatement => {
                //expected if statement token found
                //Next statement should be a relation type expression
                relation = Relation::new(tc);
            },
            err => {
                // Compiler Error :
                panic!("Expected If Statement, found unexpected Token: {:?}", err);
            },
        }

        match tc.peek_next_token_type() {
            Some(TokenType::ThenStatement) => {
                //Found Then token, consume token and move forward.
                tc.get_next_token();
                funcIfBody = FuncBody::new(tc);
            }
            None => {
                // Compiler Error :
                panic!("Unexpected end of file after if relation.");
            },
            err => {
                // Compiler Error :
                panic!("Expected Then token, found unexpected Token: {:?}", err);
            },
        }

        match tc.peek_next_token_type() {
            Some(TokenType::ElseStatement) => {
                //consume the else, pass body of statement
                tc.get_next_token();
                funcElseBody = Option::Some(FuncBody::new(tc));
            },
            Some(TokenType::FiStatement) => {
                //fall through to next match case. this is just an else handler.
            },
            None => {
                // Compiler Error :
                panic!("Unexpected end of file. Expected Else or fi statement.");
            },
            err => {
                // Compiler Error :
                panic!("Expected Else or fi statment, found unexpected Token: {:?}", err);
            },
        }

        match tc.peek_next_token_type() {
            Some(TokenType::FiStatement) => {
                tc.get_next_token();
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
                        panic!("Expected Fi statement, none was found.");
                    },
                    err => {
                        // Compiler Error :
                        panic!("Expected Fi statement, found unexpected Token: {:?}", err);
                    },
                }
            },
            err => {
                // Compiler Error :
                panic!("Expected Else or fi statment, found unexpected Token: {:?}", err);
            }
        }

        IfStmt { node_type: TokenType::IfStatement, relation, funcIfBody, funcElseBody }
    }

    pub fn get_value(&self) -> (Relation, FuncBody, Option<FuncBody>) {
        return (self.relation.clone(), self.funcIfBody.clone(), self.funcElseBody.clone())
    }

    pub fn get_type(&self) -> TokenType {
        self.node_type.clone()
    }

    pub fn to_ir(self, graph_manager: &mut GraphManager, irm: &mut IRManager) {
        /// General Order:
        /// - enter node with "central node"
        /// - create top of if-node, connect main to top
        /// - go through if-body, generate if_bottom and connect to phi node
        /// - create possible else-node, connect main to top of else
        /// - go through else-body, generate else_bottom and connect to phi node
        /// - phi node is new "central node"
        /// - go through assigned values and figure out phi

        // Clone Main Node Index + add relation statement
        let main_node = graph_manager.clone_node_index();
        self.relation.to_ir(graph_manager,irm, Value::new(ValTy::var(String::from("blank"))));

        // get current var chart.

        // Variable holder for else_node_bottom
        let mut else_node_bottom = None;

        // Generate if-node-top
        graph_manager.new_node(irm);
        let if_node_top = graph_manager.clone_node_index();
        // Connect Main Node to If-Node-Top
        graph_manager.add_edge(main_node,if_node_top);

        // Go through if-body, generate if-bottom
        self.funcIfBody.to_ir(graph_manager, irm);
        let if_node_bottom = graph_manager.clone_node_index();

        match self.funcElseBody {
            Some(funcElseBody) => {
                // Generate else-node-top
                graph_manager.new_node(irm);
                let else_node_top = graph_manager.clone_node_index();
                graph_manager.add_edge(main_node,else_node_top);

                // go through else-body, generate else-bottom
                funcElseBody.to_ir(graph_manager, irm);
                else_node_bottom = Some(graph_manager.clone_node_index());
            },
            None => {
                // Nothing to do here, fall through.
            }
        }

        // TODO : How will i get the instruction for the if to branch to?
        // TODO : Will i need a clean up cycle to determine branch locations?

        // Main branch node after if/else (phi node)
        graph_manager.new_node(irm);
        let phi_node = graph_manager.clone_node_index();

        // Figure out possible phi

        // Connect if-bottom to phi
        graph_manager.add_edge(if_node_bottom, phi_node);

        // Add else node
        match else_node_bottom {
            Some(node) => {
                // Connect else-bottom to phi
                graph_manager.add_edge(node, phi_node);
            },
            None => {
                // no else body, connect main directly to phi
                graph_manager.add_edge(main_node, phi_node);
            },
        }
    }
}