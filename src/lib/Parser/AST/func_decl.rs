use lib::Lexer::token::TokenCollection;
use lib::Lexer::token::TokenType;
use Parser::AST::func_body::FuncBody;
use Parser::AST::func_ident::FuncIdent;
use Parser::AST::var_decl::VarDecl;

use super::Graph;
use super::{IRGraphManager, InstTy, Node, NodeData, NodeId, NodeType, Op, ValTy, Value};
use super::{Rc, RefCell};
use lib::Graph::graph_manager::GraphManager;
use lib::Graph::node::NodeType::exit;
use lib::RegisterAllocator::RegisterAllocation;

#[derive(Debug, Clone)]
pub struct FuncDecl {
    node_type: TokenType,
    funcName: FuncIdent,
    varDecl: Vec<VarDecl>,
    funcBody: FuncBody,
}

impl FuncDecl {
    pub fn new(tc: &mut TokenCollection) -> Self {
        let funcName;
        let mut varDecl = vec![];
        let funcBody;

        match tc.get_next_token().expect("FuncDecl Error").get_type() {
            TokenType::FuncDecl => {
                //case matches correctly, token is consumed.
            }
            err => {
                // Compiler Error :
                panic!(
                    "Function delcaration token assumed, but not found. Found : {:?}",
                    err
                );
            }
        }

        match tc.peek_next_token_type() {
            Some(TokenType::Ident) => {
                funcName = FuncIdent::new(tc);

                match tc.peek_next_token_type() {
                    Some(TokenType::SemiTermination) => {
                        //consume Token then fall through.
                        tc.get_next_token();
                    }
                    None => {
                        // Compiler Error :
                        panic!("Expected ';' at end of function ident, but found EOF.");
                    }
                    err => {
                        // Compiler Error :
                        panic!(
                            "Expected ';' at end of func_ident, but found unexpected Token: {:?}",
                            err
                        );
                    }
                }
            }
            None => {
                // Compiler Error :
                panic!("Expected Ident Token at function declaration, found EOF.");
            }
            err => {
                // Compiler Error :
                panic!(
                    "Expected Ident Token at function declaration, found unexpected Token: {:?}",
                    err
                );
            }
        }

        while let Some(next_token) = tc.peek_next_token_type() {
            match next_token {
                TokenType::Var | TokenType::Array => {
                    varDecl.push(VarDecl::new(tc));
                }
                TokenType::LeftBrace => {
                    //do not consume, fall through
                    break;
                }
                err => {
                    // Compiler Error :
                    panic!("Expected Variable Decl or '{{' Token for start of function body, but found unexpected Token {:?}", err);
                }
            }
        }

        match tc.peek_next_token_type() {
            Some(TokenType::LeftBrace) => {
                //consume brace, call body
                tc.get_next_token();

                funcBody = FuncBody::new(tc);

                match tc.peek_next_token_type() {
                    Some(TokenType::RightBrace) => {
                        //all is well, consume token
                        tc.get_next_token();
                    }
                    None => {
                        // Compiler Error :
                        panic!("Expected '}' Token in function body, found EOF.");
                    }
                    err => {
                        // Compiler Error :
                        panic!("Expected '}}' Token at end of function body, found unexpected Token: {:?}", err);
                    }
                }
            }
            None => {
                // Compiler Error :
                panic!(
                    "Expected either variable declaration or start of function body, found EOF."
                );
            }
            err => {
                // Compiler Error :
                panic!(
                    "Expected either VarDecl Token or '{{' found unexpected: {:?}",
                    err
                );
            }
        }

        match tc.peek_next_token_type() {
            Some(TokenType::SemiTermination) => {
                //consume token, return
                tc.get_next_token();
            }
            None => {
                // Compiler Error :
                panic!("Expected ';' Token at end of function body, found EOF.");
            }
            err => {
                // Compiler Error :
                panic!(
                    "Expected ';' Token at end of function body, found unexpected Token: {:?}",
                    err
                );
            }
        }

        FuncDecl {
            node_type: TokenType::FuncDecl,
            funcName,
            varDecl,
            funcBody,
        }
    }

    pub fn get_value(&self) -> (FuncIdent, Vec<VarDecl>, FuncBody) {
        return (
            self.funcName.clone(),
            self.varDecl.to_vec(),
            self.funcBody.clone(),
        );
    }

    pub fn get_type(&self) -> TokenType {
        self.node_type.clone()
    }

    pub fn to_ir(self, irgm: &mut IRGraphManager) {
        let (func_name, func_param) = self.funcName.get_value();

        let entrance_id = irgm
            .new_node(String::from("Entrance"), NodeType::entrance)
            .clone();
        let func_index = irgm
            .new_node(func_name.get_value(), NodeType::function_head)
            .clone();
        irgm.graph_manager().add_edge(entrance_id, func_index);
        irgm.new_function(func_name.get_value(), &func_index);

        match func_param {
            Some(parameters) => {
                parameters.get_value().iter().for_each(|variable| {
                    irgm.variable_manager()
                        .active_function()
                        .add_parameter(&variable.get_value());
                    irgm.add_variable(&variable.get_value());
                });
            }
            None => {
                // Pass through
            }
        }

        // Scan function for globals used within
        self.funcBody.scan_globals(irgm);

        for var in self.varDecl {
            var.to_ir(irgm, false, Some(func_name.get_value()));
        }

        // Load all global values
        for global in irgm
            .variable_manager()
            .active_function()
            .load_globals_list()
        {
            let global_addr_val = Value::new(ValTy::adr(irgm.address_manager().get_global_reg()));
            let var_addr_val = Value::new(ValTy::adr(
                irgm.address_manager().get_addr_assignment(&global, 4),
            ));

            let adda_inst = irgm.build_op_x_y(global_addr_val, var_addr_val, InstTy::add);
            let adda_val = irgm.graph_manager().add_instruction(adda_inst);

            let inst = irgm.build_op_y(adda_val, InstTy::load);
            let inst_val = irgm.graph_manager().add_instruction(inst);

            let block_num = irgm.get_block_num();
            let inst_num = irgm.get_inst_num();

            irgm.variable_manager()
                .make_unique_variable(global, inst_val, block_num, inst_num);
        }

        // Load all param values
        for param in irgm.variable_manager().active_function().load_param_list() {
            let frame_pointer_addr =
                Value::new(ValTy::adr(irgm.address_manager().get_frame_pointer()));
            let var_addr = Value::new(ValTy::adr(
                irgm.address_manager().get_addr_assignment(&param, 4),
            ));

            let adda_inst = irgm.build_op_x_y(frame_pointer_addr, var_addr, InstTy::add);
            let adda_val = irgm.graph_manager().add_instruction(adda_inst);

            let inst = irgm.build_op_y(adda_val, InstTy::load);
            let inst_val = irgm.graph_manager().add_instruction(inst);

            let block_num = irgm.get_block_num();
            let inst_num = irgm.get_inst_num();

            irgm.variable_manager()
                .make_unique_variable(param, inst_val, block_num, inst_num);
        }

        // After loading all necessary variables, convert func_body to IR
        self.funcBody.to_ir(irgm);

        if !irgm.variable_manager().active_function().has_return() {
            // Store back all affected globals
            for global in irgm
                .variable_manager()
                .active_function()
                .load_assigned_globals()
            {
                let global_addr_val =
                    Value::new(ValTy::adr(irgm.address_manager().get_global_reg()));

                let uniq_var_val = Value::new(ValTy::var(irgm.get_current_unique(&global).clone()));
                let var_addr_val = Value::new(ValTy::adr(
                    irgm.address_manager().get_addr_assignment(&global, 4),
                ));

                let add_inst = irgm.build_op_x_y(global_addr_val, var_addr_val, InstTy::add);
                let add_reg_val = irgm.graph_manager().add_instruction(add_inst);

                let inst;
                if let ValTy::con(con_val) = uniq_var_val.clone().get_var_base().clone() {
                    let add_inst = irgm.build_op_x_y(
                        Value::new(ValTy::con(0)),
                        Value::new(ValTy::con(con_val)),
                        InstTy::add,
                    );
                    let add_val = irgm.graph_manager().add_instruction(add_inst);
                    inst = irgm.build_op_x_y(add_reg_val, add_val, InstTy::store);
                } else {
                    inst = irgm.build_op_x_y(add_reg_val, uniq_var_val, InstTy::store);
                }
                let new_global_val = irgm.graph_manager().add_instruction(inst);
            }

            // This will be a special instruction that always returns from branch location on register R31;
            let ret_inst = irgm.build_op_x(
                Value::new(ValTy::reg(RegisterAllocation::allocate_R31())),
                InstTy::ret,
            );
            irgm.graph_manager().add_instruction(ret_inst);

            // In this case there is a need to add an exit
            let current_id = irgm.graph_manager().get_current_id();
            let exit_id = irgm.new_node(String::from("Exit"), NodeType::exit).clone();
            irgm.graph_manager().add_edge(current_id, exit_id);
        }

        let uniq_func = irgm.end_function();
        irgm.function_manager().add_func_to_manager(uniq_func);
    }
}
