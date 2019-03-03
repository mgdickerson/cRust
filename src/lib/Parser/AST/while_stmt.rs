use lib::Lexer::token::TokenCollection;
use lib::Lexer::token::TokenType;
use Parser::AST::relation::Relation;
use Parser::AST::func_body::FuncBody;

use super::{Node, NodeId, NodeType, NodeData, IRGraphManager, Value, ValTy, Op, InstTy};
use super::Graph;
use super::{Rc,RefCell};
use lib::Graph::graph_manager::GraphManager;

#[derive(Debug,Clone)]
pub struct WhileStmt {
    node_type: TokenType,
    relation: Relation,
    body: FuncBody,
}

impl WhileStmt {
    pub fn new(tc: &mut TokenCollection) -> Self {
        let relation;
        let body;

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

    pub fn to_ir(self, irgm : &mut IRGraphManager) {
        /// General Order:
        /// - enters on "Main Node"
        /// - generate loop-node
        /// - connect loop head to main-node
        /// - go through loop-body, generate loop-bottom
        /// - connect loop-bottom to main-node
        /// - generate phi-node
        /// - connect main-node to phi
        /// - phi node is new "Main Node"

        // Make copy of main node
        let main_node = irgm.graph_manager().clone_node_index();

        // Make loop header
        irgm.new_node(String::from("While_Header"), NodeType::loop_header);
        let loop_header = irgm.graph_manager().clone_node_index();
        //let loop_id = irgm.graph_manager().get_node_id(loop_header);


        //let branch_id = irgm.graph_manager().get_node_id(branch_node);

        // Handy for return instruction later
        irgm.graph_manager().switch_current_node_index(loop_header.clone());
        self.relation.to_ir(irgm, Value::new(ValTy::con(-1)));

        irgm.graph_manager().add_edge(main_node, loop_header);
        let main_vars = irgm.variable_manager().var_checkpoint();

        // Generate loop-body head
        irgm.new_node(String::from("Loop_Head"), NodeType::while_node);
        let loop_node_top = irgm.graph_manager().clone_node_index();
        // Connect main_node to loop_node_top
        irgm.graph_manager().add_edge(loop_header,loop_node_top);

        // Go through loop body
        self.body.to_ir(irgm);
        // Add return branch instruction to "new main node"
        let bra_return = irgm.build_op_y(Value::new(ValTy::node_id(loop_header.clone())),InstTy::bra);
        irgm.graph_manager().add_instruction(bra_return);
        let loop_vars = irgm.variable_manager().var_checkpoint();

        irgm.variable_manager().restore_vars(main_vars.clone());

        // Generate new loop bottom node
        let loop_node_bottom = irgm.graph_manager().clone_node_index();

        // Generate phi node
        irgm.new_node(String::from("Bra_Node"), NodeType::phi_node);
        let branch_node = irgm.graph_manager().clone_node_index();

        irgm.graph_manager().add_edge(loop_header,branch_node);
        irgm.graph_manager().add_edge(loop_node_bottom,loop_header);

        // Insert Phi Inst to Loop Header
        irgm.graph_manager().switch_current_node_index(loop_header);

        // Update final instruction in loop header to point to newly created phi_node
        irgm.graph_manager().get_mut_ref_graph().node_weight_mut(loop_header)
            .unwrap().get_mut_data_ref()
            .get_inst_list_ref()
            .last().unwrap()
            .borrow_mut()
            .update_y_val(
                Value::new(ValTy::node_id(branch_node)
                ));

        let changed_vars = irgm.insert_phi_inst(loop_vars, main_vars);
        let uses_to_remove = irgm.loop_variable_correction(changed_vars);

        irgm.graph_manager().switch_current_node_index(branch_node);
    }

    pub fn scan_globals(&self, irgm : &mut IRGraphManager) {
        self.relation.scan_globals(irgm);
        self.body.scan_globals(irgm);
    }
}