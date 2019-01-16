use lib::Lexer::token::TokenCollection;
use lib::Lexer::token::TokenType;
use Parser::AST::func_ident::FuncIdent;
use Parser::AST::func_body::FuncBody;
use Parser::AST::var_decl::VarDecl;

use super::{Node, NodeId, NodeData, IRManager, Value, ValTy, Op, InstTy};
use super::Graph;

#[derive(Debug,Clone)]
pub struct FuncDecl {
    node_type: TokenType,
    funcName: FuncIdent,
    varDecl: Vec<VarDecl>,
    funcBody: FuncBody,
}

impl FuncDecl {
    pub fn new(tc: &mut TokenCollection) -> Self {
        let mut funcName;
        let mut varDecl = vec!();
        let mut funcBody;

        match tc.get_next_token().expect("FuncDecl Error").get_type() {
            TokenType::FuncDecl => {
                //case matches correctly, token is consumed.
            },
            err => {
                // Compiler Error :
                panic!("Function delcaration token assumed, but not found. Found : {:?}", err);
            },
        }

        match tc.peek_next_token_type() {
            Some(TokenType::Ident) => {
                funcName = FuncIdent::new(tc);

                match tc.peek_next_token_type() {
                    Some(TokenType::SemiTermination) => {
                        //consume Token then fall through.
                        tc.get_next_token();
                    },
                    None => {
                        // Compiler Error :
                        panic!("Expected ';' at end of function ident, but found EOF.");
                    },
                    err => {
                        // Compiler Error :
                        panic!("Expected ';' at end of func_ident, but found unexpected Token: {:?}", err);
                    },
                }
            },
            None => {
                // Compiler Error :
                panic!("Expected Ident Token at function declaration, found EOF.");
            },
            err => {
                // Compiler Error :
                panic!("Expected Ident Token at function declaration, found unexpected Token: {:?}", err);
            },
        }

        while let Some(next_token) = tc.peek_next_token_type() {
            match next_token {
                TokenType::Var | TokenType::Array => {
                    varDecl.push(VarDecl::new(tc));
                },
                TokenType::LeftBrace => {
                    //do not consume, fall through
                    break;
                },
                err => {
                    // Compiler Error :
                    panic!("Expected Variable Decl or '{{' Token for start of function body, but found unexpected Token {:?}", err);
                },
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
                    },
                    None => {
                        // Compiler Error :
                        panic!("Expected '}' Token in function body, found EOF.");
                    },
                    err => {
                        // Compiler Error :
                        panic!("Expected '}}' Token at end of function body, found unexpected Token: {:?}", err);
                    },
                }
            },
            None => {
                // Compiler Error :
                panic!("Expected either variable declaration or start of function body, found EOF.");
            },
            err => {
                // Compiler Error :
                panic!("Expected either VarDecl Token or '{{' found unexpected: {:?}", err);
            },
        }

        match tc.peek_next_token_type() {
            Some(TokenType::SemiTermination) => {
                //consume token, return
                tc.get_next_token();
            },
            None => {
                // Compiler Error :
                panic!("Expected ';' Token at end of function body, found EOF.");
            },
            err => {
                // Compiler Error :
                panic!("Expected ';' Token at end of function body, found unexpected Token: {:?}", err);
            },
        }

        FuncDecl { node_type: TokenType::FuncDecl, funcName, varDecl, funcBody }
    }

    pub fn get_value(&self) -> (FuncIdent, Vec<VarDecl>, FuncBody)  {
        return (self.funcName.clone(), self.varDecl.to_vec(), self.funcBody.clone())
    }

    pub fn get_type(&self) -> TokenType {
        self.node_type.clone()
    }

    pub fn to_ir(self, graph: &mut Graph<Node, i32>, current_node: &mut Node, irm: &mut IRManager) {
        let (func_name, func_param) = self.funcName.get_value();
        let func_name_string = String::from("f_") + &func_name.get_value();

        irm.get_var_manager_mut_ref().add_variable(func_name_string.clone());
        match func_param {
            Some(param) => {
                for p in param.get_value() {
                    let param_ident = String::from("param_") + &p.get_value();
                    if irm.get_var_manager_mut_ref().is_valid_variable(param_ident.clone()) {
                        // this variable is already a global variable, send error.
                        panic!("{} local variable {} is already a global variable.", func_name_string.clone(), param_ident.clone());
                    }

                    let func_param_name = func_name_string.clone() + "_" + &param_ident;
                    irm.get_var_manager_mut_ref().add_variable(func_param_name);
                }
            },
            None => {
                // Fall through
            },
        }

        for var in self.varDecl {
            var.to_ir(graph, current_node,irm,false,Some(func_name_string.clone()));
        }


    }
}