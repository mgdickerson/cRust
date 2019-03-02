use lib::Lexer::token::TokenCollection;
use lib::Lexer::token::TokenType;
use Parser::AST::ident::Ident;
use Parser::AST::expression::Expression;

use lib::IR::ret_register::RetRegister;
use super::{Node, NodeId, NodeData, IRGraphManager, Value, ValTy, Op, InstTy};
use super::Graph;
use super::{Rc,RefCell};
use lib::Parser::AST::factor::FactorType::expr;

#[derive(Debug,Clone)]
pub struct FuncCall {
    node_type: TokenType,
    funcName: Ident,
    variables: Vec<Expression>,
}

impl FuncCall {
    pub fn new(tc: &mut TokenCollection) -> Self {
        let mut variables = vec!();
        let funcName;

        match tc.get_next_token().expect("Func Call Error").get_type() {
            TokenType::FuncCall => {
                //this is as was expected, call function ident;

                //check for function identity.
                match tc.peek_next_token_type() {
                    Some(TokenType::Ident) => {
                        funcName = Ident::new(tc);   //this will grab function identity AND parameters.

                        match tc.peek_next_token_type() {
                            Some(TokenType::LeftPara) => {
                                //function parameter start
                                tc.get_next_token();

                                while let Some(next_token) = tc.peek_next_token_type() {
                                    match next_token {
                                        TokenType::RightPara => {
                                            tc.get_next_token();
                                            break;
                                        },
                                        TokenType::Comma => {
                                            //consume token and get next expr
                                            tc.get_next_token();
                                            variables.push(Expression::new(tc));
                                        },
                                        exprs => {
                                            //get next expression
                                            variables.push(Expression::new(tc));
                                        },
                                        TokenType::SemiTermination => {
                                            panic!("Expected variables, or right brace, found ';'.");
                                        }
                                    }
                                }
                            },
                            None => {
                                // Compiler Error :
                                panic!("Expected anything after func_call, found EOF.");
                            },
                            ret => {
                                //it was literally any other case, so just return and handle elsewhere
                            },
                        }
                        // TODO :
                        //here we add to some table with function declaration and function call.
                        //depending on the table, we could declare a function after it is used similar to rust.
                        //this would require meta-data and unwinding possible errors in functions
                        //not existing.
                    },
                    None => {
                        // Compiler Error :
                        panic!("Expected Function Identity, found end of file.");
                    },
                    err => {
                        // Compiler Error :
                        panic!("Expected Function Identity, found unexpected Token: {:?}", err);
                    },
                }
            },
            err => {
                // Compiler Error :
                panic!("Expected Functional Call Token, found unexpected Token: {:?}", err);
            },
        }

        //can probably just return after this
        FuncCall { node_type: TokenType::FuncCall, funcName, variables }
    }

    pub fn get_value(&self) -> (Ident, Vec<Expression>)  {
        return (self.funcName.clone(), self.variables.to_vec())
    }

    pub fn get_type(&self) -> TokenType {
        self.node_type.clone()
    }

    pub fn to_ir(self, irgm: &mut IRGraphManager) -> Option<Value> {
        match self.funcName.get_value().as_ref() {
            "InputNum" => {
                let inp_num = String::from("read");
                let inst = irgm.build_spec_op(&inp_num, InstTy::read);
                let inst_val = irgm.graph_manager().add_instruction(inst);
                return Some(inst_val);
            },
            "OutputNum" => {
                if self.variables.len() > 1 {
                    panic!("There should only be 1 variable in OutputNum.");
                }

                let mut expr_val = self.variables.last().expect("There should be at least one argument.").to_owned().to_ir(irgm).expect("Should contain return value.");
                if let ValTy::con(con) = expr_val.get_value().clone() {
                    let add_inst = irgm.build_op_x_y(Value::new(ValTy::con(0)), Value::new(ValTy::con(con)), InstTy::add);
                    expr_val = irgm.graph_manager().add_instruction(add_inst);
                }
                let inst = irgm.build_op_x(expr_val, InstTy::write);
                return Some(irgm.graph_manager().add_instruction(inst));
            },
            "OutputNewLine" => {
                let new_line = String::from("writeNL");
                let inst = irgm.build_spec_op(&new_line, InstTy::writeNL);
                irgm.graph_manager().add_instruction(inst);
            },
            func_name => {
                let uniq_func = irgm.get_func_call(&String::from(func_name));

                //println!("{} : \tGlobals {:?}\n\tParams: {:?}", func_name, &uniq_func.load_globals_list(), &uniq_func.load_param_list());

                // Store all global parameters affected.
                for global in &uniq_func.load_globals_list() {
                    let global_addr_val = Value::new(ValTy::adr(irgm.address_manager().get_global_reg()));

                    let uniq_var_val = Value::new(ValTy::var(irgm.get_current_unique(global).clone()));
                    let var_addr_val = Value::new(ValTy::adr(irgm.address_manager().get_addr_assignment(global, 4)));

                    let add_inst = irgm.build_op_x_y(global_addr_val, var_addr_val, InstTy::add);
                    let add_reg_val = irgm.graph_manager().add_instruction(add_inst);

                    let inst;
                    if let ValTy::con(con_val) = uniq_var_val.clone().get_var_base().clone() {
                        let add_inst = irgm.build_op_x_y(Value::new(ValTy::con(0)), Value::new(ValTy::con(con_val)), InstTy::add);
                        let add_val = irgm.graph_manager().add_instruction(add_inst);
                        inst = irgm.build_op_x_y(add_reg_val, add_val, InstTy::store);
                    } else {
                        inst = irgm.build_op_x_y(add_reg_val, uniq_var_val, InstTy::store);
                    }
                    irgm.graph_manager().add_instruction(inst);
                }

                // Store all called parameters affected.
                for (count, param) in uniq_func.load_param_list().iter().enumerate() {
                    let param_addr_val = Value::new(ValTy::adr(irgm.address_manager().get_frame_pointer()));

                    // Unlike global this will pull value from the vec<expr> contained, not pull from list.
                    let uniq_var_val;
                    if count < self.variables.len() {
                        uniq_var_val = self.variables[count].to_owned().to_ir(irgm).expect("All called variables should have some expr.");
                    } else {
                        uniq_var_val = Value::new(ValTy::con(0));
                    }

                    let var_addr_val = Value::new(ValTy::adr(irgm.address_manager().get_addr_assignment(param, 4)));

                    let add_inst = irgm.build_op_x_y(param_addr_val, var_addr_val, InstTy::add);
                    let add_reg_val = irgm.graph_manager().add_instruction(add_inst);

                    let inst;
                    if let ValTy::con(con_val) = uniq_var_val.clone().get_var_base().clone() {
                        let add_inst = irgm.build_op_x_y(Value::new(ValTy::con(0)), Value::new(ValTy::con(con_val)), InstTy::add);
                        let add_val = irgm.graph_manager().add_instruction(add_inst);
                        inst = irgm.build_op_x_y(add_reg_val, add_val, InstTy::store);
                    } else {
                        inst = irgm.build_op_x_y(add_reg_val, uniq_var_val, InstTy::store);
                    }
                    irgm.graph_manager().add_instruction(inst);
                }

                // All variables have been loaded, call function
                let inst = irgm.build_spec_op(&func_name.to_string(), InstTy::call);
                irgm.graph_manager().add_instruction(inst);

                // Then I need to load back all the affected globals.
                for global in &uniq_func.load_assigned_globals() {
                    let global_addr_val = Value::new(ValTy::adr(irgm.address_manager().get_global_reg()));
                    let var_addr_val = Value::new(ValTy::adr(irgm.address_manager().get_addr_assignment(global, 4)));

                    let add_inst = irgm.build_op_x_y(global_addr_val, var_addr_val, InstTy::add);
                    let add_reg_val = irgm.graph_manager().add_instruction(add_inst);

                    let inst = irgm.build_op_y(add_reg_val, InstTy::load);
                    let new_global_val = irgm.graph_manager().add_instruction(inst);

                    let block_num = irgm.get_block_num();
                    let inst_num = irgm.get_inst_num() + 1;
                    irgm.variable_manager().make_unique_variable(global.clone(), new_global_val, block_num, inst_num);
                }

                // println!("Called function {} has return: {}", func_name, uniq_func.has_return());
                if uniq_func.has_return() {
                    return Some(Value::new(ValTy::ret(RetRegister::new())));
                }
            },
        }

        // If there is not an associated return type, return None
        None
    }

    pub fn scan_globals(&self, irgm : &mut IRGraphManager) {
        let recursive_call = irgm.variable_manager().active_function().get_name();

        if self.funcName.get_value() == recursive_call {
            for expr_var in &self.variables {
                expr_var.scan_globals(irgm);
            }

            // There are no further global items this should call
            return
        }

        match self.funcName.get_value().as_ref() {
            "InputNum" => {},
            "OutputNum" => {},
            "OutputNewLine" => {},
            func_name => {
                //println!("{} calls {}", func_name, irgm.function_manager().get_mut_function())
                let affected_globals = irgm.function_manager().get_mut_function(&self.funcName.get_value()).load_globals_list();
                for global in affected_globals {
                    irgm.variable_manager().active_function().add_global(&global);
                }

                let assigned_globals = irgm.function_manager().get_mut_function(&self.funcName.get_value()).load_assigned_globals();
                for assigned_global in assigned_globals {
                    irgm.variable_manager().active_function().add_assigned_global(&assigned_global);
                }
            },
        }

        for expr_var in &self.variables {
            expr_var.scan_globals(irgm);
        }
    }
}