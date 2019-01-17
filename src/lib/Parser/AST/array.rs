use lib::Lexer::token::TokenCollection;
use lib::Lexer::token::TokenType;
use Parser::AST;

use super::{Node, NodeId, NodeData, IRManager, Value, ValTy, Op, InstTy};
use super::Graph;

#[derive(Debug,Clone)]
pub struct Array {
    node_type: TokenType,
    arrayDepthVec: Vec<AST::number::Number>,
    identList: Vec<AST::ident::Ident>,
    debugLine: String,
}

impl Array {
    pub fn new(tc: &mut TokenCollection) -> Self {
        let mut varList: Vec<AST::ident::Ident> = vec!();
        let mut numList: Vec<AST::number::Number> = vec!();
        let mut tokenType = TokenType::None;

        match tc.get_next_token().expect("Array Error").get_type() {
            TokenType::Array => {
                // proper action, all is well.
                tokenType = TokenType::Array;
            },
            err => {
                // Compiler Error : 
                panic!("Expected Array declaration, found unexpected Token: {:?}", err);
            },
        }

        match tc.peek_next_token_type() {
            Some(TokenType::LeftBracket) => {
                //all is well, proceed through
            },
            None => {
                // Compiler Error : 
                panic!("Expected Array '[' Token, found EOF.");
            },
            err => {
                // Compiler Error : 
                panic!("Expected '[' Token in array declaration, found unexpected Token: {:?}", err);
            },
        }

        while let Some(next_token) = tc.peek_next_token_type() {
            match next_token {
                TokenType::LeftBracket => {
                    tc.get_next_token();

                    match tc.peek_next_token_type() {
                        Some(TokenType::Number) => {
                            numList.push(AST::number::Number::new(tc));
                        },
                        None => {
                            // Compiler Error : 
                            panic!("Expected Array index Number, found EOF.");
                        },
                        err => {
                            // Compiler Error : 
                            panic!("Expected Number Token in array index, found unexpected Token: {:?}", err);
                        },
                    }

                    match tc.peek_next_token_type() {
                        Some(TokenType::RightBracket) => {
                            tc.get_next_token();
                        },
                        None => {
                            // Compiler Error : 
                            panic!("Expected Array ']', found EOF.");
                        },
                        err => {
                            // Compiler Error : 
                            panic!("Expected ']' Token in array declaration, found unexpected Token: {:?}", err);
                        },
                    }
                },
                TokenType::Ident => {
                    // first ident case found, break from loop. 
                    break;
                }
                err => {
                    // Compiler Error : 
                    panic!("Expected Array T: {:?}", err);
                },
            }
        }

        while let Some(next_token) = tc.peek_next_token_type() {
            match next_token {
                TokenType::Ident => {
                    varList.push(AST::ident::Ident::new(tc));
                },
                TokenType::Comma => {
                    tc.get_next_token();
                    continue;
                }
                TokenType::SemiTermination => {
                    tc.get_next_token();
                    break;
                }
                err => {
                    // Compiler Error : 
                    panic!("Expected Ident Token in array declaration, found unexpected Token: {:?}", err);
                },
            }
        }

        Array {
            node_type: tokenType,
            arrayDepthVec: numList,
            identList: varList,
            debugLine: String::from("test")
        }
    }

    pub fn get_value(&self) -> (Vec<AST::number::Number>, Vec<AST::ident::Ident>) {
        return (self.arrayDepthVec.to_vec(), self.identList.to_vec())
    }

    pub fn get_type(&self) -> TokenType {
        self.node_type.clone()
    }

    pub fn get_debug(self) -> String {
        self.debugLine.clone()
    }

    // TODO : Currently the vector depth information is tossed, will need to handle this in var handler
    pub fn to_ir(self, graph: &mut Graph<Node, i32>, current_node: &mut Node, irm: &mut IRManager, is_global: bool, func_name: Option<String>) {
        for ident in self.identList {
            let mut var_name = ident.get_value();

            if !is_global {
                if irm.get_var_manager_mut_ref().is_valid_variable(var_name.clone()) {
                    // this variable is already a global variable, send error.
                    panic!("{} local variable {} is already a global variable.", func_name.unwrap().clone(), var_name);
                }

                var_name = func_name.clone().unwrap() + "_" + &var_name;
            }

            let unique = Array::get_unique(var_name, irm);

            let inst = irm.build_op_x_y(Value::new(ValTy::var(unique)), Value::new(ValTy::con(0)), InstTy::mov);
            current_node.get_mut_data_ref().add_instruction(inst);
        }
    }

    fn get_unique(var: String, irm: &mut IRManager) -> String {
        let unique = irm.add_variable(var);
        unique.get_ident()
    }
}