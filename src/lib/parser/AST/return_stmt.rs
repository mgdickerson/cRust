use lib::Lexer::token::TokenCollection;
use lib::Lexer::token::TokenType;
use Parser::AST::expression::Expression;

use super::Graph;
use super::{IRGraphManager, InstTy, Node, NodeData, NodeId, Op, ValTy, Value};
use lib::Graph::node::NodeType;
use lib::Graph::node::NodeType::exit;
use lib::RegisterAllocator::RegisterAllocation;
use lib::IR::address_manager::AddressType;

#[derive(Debug, Clone)]
pub struct ReturnStmt {
    node_type: TokenType,
    expression: Expression,
}

impl ReturnStmt {
    pub fn new(tc: &mut TokenCollection) -> Self {
        match tc
            .get_next_token()
            .expect("Return Statement Error")
            .get_type()
        {
            TokenType::ReturnStatement => {
                // return token found, pass through to handle expression.
                // Otherwise, error handle.
            }
            // TODO : fix up to proper error handler
            err => {
                println!(
                    "Expected Return Statement, found unexpected Token: {:?}",
                    err
                );
            } //proper method of error handling unexpected tokens
        }

        let expression = Expression::new(tc);

        match tc.peek_next_token_type() {
            Some(TokenType::SemiTermination) => {
                //Found ';' so there are likely to be more statements. Consume and return.
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
                panic!("End of file found, do should be appended by '}' if end of statement");
            }
            err => {
                // Compiler Error :
                panic!(
                    "Expected to find ';' or end  statement, found unexpected Token: {:?}",
                    err
                );
            }
        }

        ReturnStmt {
            node_type: TokenType::ReturnStatement,
            expression,
        }
    }

    pub fn get_value(&self) -> Expression {
        return self.expression.clone();
    }

    pub fn get_type(&self) -> TokenType {
        self.node_type.clone()
    }

    pub fn to_ir(self, irgm: &mut IRGraphManager) {
        let ret_val = self.expression.to_ir(irgm);

        // Store back all affected globals
        for global in irgm
            .variable_manager()
            .active_function()
            .load_assigned_globals()
        {
            let uniq_var_val = Value::new(ValTy::var(irgm.get_current_unique(&global).clone()));
            let var_addr_val = Value::new(ValTy::adr(
                irgm.address_manager().get_addr_assignment(&global, AddressType::global_var, 4, None),
            ));

            let inst;
            if let ValTy::con(con_val) = uniq_var_val.clone().get_var_base().clone() {
                let add_inst = irgm.build_op_x_y(
                    Value::new(ValTy::con(0)),
                    Value::new(ValTy::con(con_val)),
                    InstTy::add,
                );
                let add_val = irgm.graph_manager().add_instruction(add_inst);
                inst = irgm.build_op_x_y(var_addr_val, add_val, InstTy::gstore);
            } else {
                inst = irgm.build_op_x_y(var_addr_val, uniq_var_val, InstTy::gstore);
            }
            let new_global_val = irgm.graph_manager().add_instruction(inst);
        }

        let mut func_name = None;
        if let Some(func) = irgm.variable_manager().check_active_function() {
            func_name = Some(func.get_name());
        }

        // If unique function has a return, pre-load space for a return.
        let return_address_val = Value::new(ValTy::adr(
            irgm.address_manager().get_addr_assignment(&String::from("return"), AddressType::local_var, 4, func_name),
        ));

        let store_inst = irgm.build_op_x_y(
            return_address_val,
            ret_val.expect("return calls should always return an expr"),
            InstTy::gstore);
        irgm.graph_manager().add_instruction(store_inst);

        // This will be a special instruction that always returns from branch location on register R31;
        let ret_inst = irgm.build_op_x(
            Value::new(ValTy::reg(RegisterAllocation::allocate_R31())),
            InstTy::ret,
        );
        irgm.graph_manager().add_instruction(ret_inst);

        // Create an exit at this point
        let current_node_id = irgm
            .graph_manager()
            .get_mut_ref_current_node_index()
            .clone();
        let exit_id = irgm.new_node(String::from("Exit"), NodeType::exit).clone();
        irgm.graph_manager()
            .add_edge(current_node_id, exit_id.clone());

        let ignored_id = irgm
            .new_node(String::from("Ignored"), NodeType::ignored)
            .clone();
        irgm.graph_manager().add_edge(exit_id, ignored_id);
    }

    pub fn scan_globals(&self, irgm: &mut IRGraphManager) {
        irgm.variable_manager().active_function().set_return(true);
        self.expression.scan_globals(irgm);
    }
}
