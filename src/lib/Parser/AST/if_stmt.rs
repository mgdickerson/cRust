use lib::Lexer::token::TokenCollection;
use lib::Lexer::token::TokenType;
use Parser::AST::func_body::FuncBody;
use Parser::AST::relation::Relation;

use super::Graph;
use super::{IRGraphManager, InstTy, Node, NodeData, NodeId, NodeType, Op, ValTy, Value};
use lib::Graph::graph_manager::GraphManager;

#[derive(Debug, Clone)]
pub struct IfStmt {
    node_type: TokenType,
    relation: Relation,
    funcIfBody: FuncBody,
    funcElseBody: Option<FuncBody>,
}

impl IfStmt {
    pub fn new(tc: &mut TokenCollection) -> Self {
        let relation;
        let funcIfBody;
        let mut funcElseBody = Option::None;

        match tc.get_next_token().expect("If Statment Error").get_type() {
            TokenType::IfStatement => {
                //expected if statement token found
                //Next statement should be a relation type expression
                relation = Relation::new(tc);
            }
            err => {
                // Compiler Error :
                panic!("Expected If Statement, found unexpected Token: {:?}", err);
            }
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
            }
            err => {
                // Compiler Error :
                panic!("Expected Then token, found unexpected Token: {:?}", err);
            }
        }

        match tc.peek_next_token_type() {
            Some(TokenType::ElseStatement) => {
                //consume the else, pass body of statement
                tc.get_next_token();
                funcElseBody = Option::Some(FuncBody::new(tc));
            }
            Some(TokenType::FiStatement) => {
                //fall through to next match case. this is just an else handler.
            }
            None => {
                // Compiler Error :
                panic!("Unexpected end of file. Expected Else or fi statement.");
            }
            err => {
                // Compiler Error :
                panic!(
                    "Expected Else or fi statment, found unexpected Token: {:?}",
                    err
                );
            }
        }

        match tc.peek_next_token_type() {
            Some(TokenType::FiStatement) => {
                tc.get_next_token();
                match tc.peek_next_token_type() {
                    Some(TokenType::SemiTermination) => {
                        //consume token, return.
                        tc.get_next_token();
                    }
                    // All Possible Ending Sequences where ';' may not be necessary.
                    Some(TokenType::FiStatement)
                    | Some(TokenType::OdStatement)
                    | Some(TokenType::RightBrace)
                    | Some(TokenType::ElseStatement) => {
                        //';' not required, return without consuming token.
                    }
                    None => {
                        // Compiler Error :
                        panic!("Expected Fi statement, none was found.");
                    }
                    err => {
                        // Compiler Error :
                        panic!("Expected Fi statement, found unexpected Token: {:?}", err);
                    }
                }
            }
            err => {
                // Compiler Error :
                panic!(
                    "Expected Else or fi statment, found unexpected Token: {:?}",
                    err
                );
            }
        }

        IfStmt {
            node_type: TokenType::IfStatement,
            relation,
            funcIfBody,
            funcElseBody,
        }
    }

    pub fn get_value(&self) -> (Relation, FuncBody, Option<FuncBody>) {
        return (
            self.relation.clone(),
            self.funcIfBody.clone(),
            self.funcElseBody.clone(),
        );
    }

    pub fn get_type(&self) -> TokenType {
        self.node_type.clone()
    }

    pub fn to_ir(self, irgm: &mut IRGraphManager) {
        /// General Order:
        /// - enter node with "central node"
        /// - create top of if-node, connect main to top
        /// - go through if-body, generate if_bottom and connect to phi node
        /// - create possible else-node, connect main to top of else
        /// - go through else-body, generate else_bottom and connect to phi node
        /// - phi node is new "central node"
        /// - go through assigned values and figure out phi
        // Clone Main Node Index + add relation statement
        let main_node = irgm.graph_manager().clone_node_index();

        irgm.new_node(String::from("If_Header"), NodeType::if_header);
        let loop_header = irgm.graph_manager().clone_node_index();
        irgm.graph_manager().add_edge(main_node, loop_header);

        let main_checkpoint = irgm.variable_manager().var_checkpoint();

        // Variable holder for else_node_bottom
        let mut else_node_top = None;
        let mut else_node_bottom = None;
        let mut else_checkpoint = None;

        // Generate if-node-top
        irgm.new_node(String::from("If_Top_Node"), NodeType::if_node);
        let if_node_top = irgm.graph_manager().clone_node_index();
        // Connect Main Node to If-Node-Top
        irgm.graph_manager().add_edge(loop_header, if_node_top);

        // Go through if-body, generate if-bottom
        self.funcIfBody.to_ir(irgm);

        let if_node_bottom = irgm.graph_manager().clone_node_index();

        let if_checkpoint = irgm.variable_manager().var_checkpoint();

        match self.funcElseBody {
            Some(funcElseBody) => {
                // There is an else branch, so it is important to add the branch to the if node.
                let if_branch = irgm.build_op_y(Value::new(ValTy::con(-1)), InstTy::bra);
                irgm.graph_manager().add_instruction(if_branch);

                irgm.variable_manager()
                    .restore_vars(main_checkpoint.clone());

                // Generate else-node-top
                irgm.new_node(String::from("Else_Top_Node"), NodeType::else_node);
                else_node_top = Some(irgm.graph_manager().clone_node_index());
                irgm.graph_manager()
                    .add_edge(loop_header, else_node_top.clone().unwrap());

                // go through else-body, generate else-bottom
                funcElseBody.to_ir(irgm);
                else_node_bottom = Some(irgm.graph_manager().clone_node_index());

                let check = irgm.variable_manager().var_checkpoint();
                else_checkpoint = Some(check);
            }
            None => {
                // Nothing to do here, fall through.
            }
        }

        // Main branch node after if/else (phi node)
        irgm.new_node(String::from("Phi_Node"), NodeType::phi_node);
        let phi_node = irgm.graph_manager().clone_node_index();

        // Connect branch command to phi node
        if let Some(else_body) = &else_node_top {
            irgm.graph_manager()
                .get_mut_ref_graph()
                .node_weight_mut(if_node_bottom)
                .unwrap()
                .get_mut_data_ref()
                .get_inst_list_ref()
                .last()
                .unwrap()
                .borrow_mut()
                .update_y_val(Value::new(ValTy::node_id(phi_node)));
        }

        // Connect if-bottom to phi
        irgm.graph_manager().add_edge(if_node_bottom, phi_node);

        irgm.variable_manager()
            .restore_vars(main_checkpoint.clone());
        // Add else node
        match else_node_bottom {
            Some(node) => {
                irgm.graph_manager().switch_current_node_index(loop_header);
                //let else_id = irgm.graph_manager().get_node_id(else_node_top.unwrap());
                self.relation
                    .to_ir(irgm, Value::new(ValTy::node_id(else_node_top.unwrap())));
                irgm.graph_manager().switch_current_node_index(phi_node);

                // Connect else-bottom to phi
                irgm.graph_manager().add_edge(node, phi_node);

                // Construct phi by checking first if and else
                // If they differ, construct phi out of both.

                irgm.insert_phi_inst(
                    if_checkpoint,
                    else_checkpoint
                        .expect("There is an else node, there should be an else checkpoint."),
                );
            }
            None => {
                irgm.graph_manager().switch_current_node_index(loop_header);
                //let phi_id = irgm.graph_manager().get_node_id(phi_node.clone());
                self.relation
                    .to_ir(irgm, Value::new(ValTy::node_id(phi_node.clone())));
                irgm.graph_manager().switch_current_node_index(phi_node);

                // no else body, connect main directly to phi
                irgm.graph_manager().add_edge(loop_header, phi_node);

                irgm.insert_phi_inst(if_checkpoint, main_checkpoint);
            }
        }
    }

    pub fn scan_globals(&self, irgm: &mut IRGraphManager) {
        self.relation.scan_globals(irgm);
        self.funcIfBody.scan_globals(irgm);
        match &self.funcElseBody {
            Some(else_body) => {
                else_body.scan_globals(irgm);
            }
            None => {
                // Do Nothing
            }
        }
    }
}
