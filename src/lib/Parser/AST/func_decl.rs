use lib::Lexer::token::TokenCollection;
use lib::Lexer::token::TokenType;
use Parser::AST::func_ident::FuncIdent;
use Parser::AST::func_body::FuncBody;
use Parser::AST::var_decl::VarDecl;

use super::{Node, NodeType, NodeId, NodeData, IRGraphManager, Value, ValTy, Op, InstTy};
use super::Graph;
use lib::Graph::graph_manager::GraphManager;

#[derive(Debug,Clone)]
pub struct FuncDecl {
    node_type: TokenType,
    funcName: FuncIdent,
    varDecl: Vec<VarDecl>,
    funcBody: FuncBody,
}

impl FuncDecl {
    pub fn new(tc: &mut TokenCollection) -> Self {
        let funcName;
        let mut varDecl = vec!();
        let funcBody;

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

    pub fn to_ir(self, irgm : &mut IRGraphManager) {
        let (func_name, func_param) = self.funcName.get_value();

        irgm.new_function(func_name.get_value());

        let block_num = irgm.get_block_num();
        let inst_num = irgm.get_inst_num();

        match func_param {
            Some(parameters) => {
                parameters.get_value()
                    .iter()
                    .for_each(|variable| {
                        irgm.variable_manager().add_parameters(variable.get_value(), block_num, inst_num);
                    });
            },
            None => {
                // Pass through
            },
        }

        for var in self.varDecl {
            var.to_ir(irgm, false, Some(func_name.get_value()));
        }

        irgm.new_node(func_name.get_value(), NodeType::function_head);
        self.funcBody.to_ir(irgm);

        let uniq_func = irgm.end_function();
        irgm.function_manager().add_func_to_manager(uniq_func);
    }
}