use lib::Lexer::token::TokenCollection;
use lib::Lexer::token::TokenType;
use Parser::AST;

use super::{Node, NodeId, NodeData, IRManager, Value, ValTy, Op, InstTy};
use super::Graph;

#[derive(Debug,Clone)]
pub struct Var {
    var_type: TokenType,
    var_vec: Vec<AST::ident::Ident>,
    debugLine: String,
}

impl Var {
    pub fn new(tc: &mut TokenCollection) -> Self {
        let mut varList : Vec<AST::ident::Ident> = vec!();
        let mut varTokenType = TokenType::None;

        match tc.get_next_token().expect("Var Error").get_type() {
            TokenType::Var => {
                //This is accepted behavior, pass through.
                varTokenType = TokenType::Var;
            },
            err => {
                // Compiler Error : 
                panic!("Expected Variable declaration, found unexpected Token: {:?}", err);
            }
        }

        while let Some(next_token) = tc.peek_next_token_type() {
            match next_token {
                TokenType::Ident => {
                    varList.push(AST::ident::Ident::new(tc));
                },
                TokenType::Comma => {
                    //consume comma token
                    tc.get_next_token();
                },
                TokenType::SemiTermination => {
                    //consume semicolon and return. 
                    tc.get_next_token();
                    break;
                },
                err => {
                    // Compiler Error : 
                    panic!("Unable to parse token in variable declaration: {:?}", err);
                },
            }
        }

        Var{ var_type: varTokenType, var_vec: varList, debugLine: String::from("test")}
    }

    pub fn get_value(&self) -> Vec<AST::ident::Ident> {
        self.var_vec.to_vec()
    }

    pub fn get_type(&self) -> TokenType {
        self.var_type.clone()
    }

    pub fn get_debug(self) -> String {
        self.debugLine.clone()
    }

    pub fn to_ir(self, current_node: &mut Node, irm: &mut IRManager, is_global: bool, func_name: Option<String>) {
        for ident in self.var_vec {
            let mut var_name = ident.get_value();

            if !is_global {
                if irm.get_var_manager_mut_ref().is_valid_variable(var_name.clone()) {
                    // this variable is already a global variable, send error.
                    panic!("{} local variable {} is already a global variable.", func_name.unwrap().clone(), var_name);
                }

                var_name = func_name.clone().unwrap() + "_" + &var_name;
            }

            let unique = Var::get_unique(var_name, irm);

            //let inst = irm.build_op_x_y(Value::new(ValTy::var(unique)), Value::new(ValTy::con(0)), InstTy::mov);
            //current_node.get_mut_data_ref().add_instruction(inst);
        }
    }

    fn get_unique(var: String, irm: &mut IRManager) -> String {
        let unique = irm.add_variable(var, Value::new(ValTy::con(0)));
        unique.get_ident()
    }
}