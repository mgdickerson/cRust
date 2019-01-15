use lib::Lexer::token::TokenCollection;
use lib::Lexer::token::TokenType;

use Parser::AST::var_decl::VarDecl;
use Parser::AST::func_decl::FuncDecl;
use Parser::AST::func_body::FuncBody;

use super::{Node, NodeId, NodeData, IRManager, Value, ValTy, Op, InstTy};
use super::Graph;

#[derive(Debug,Clone)]
pub struct Comp {
    node_type: TokenType,
    varDecl: Vec<VarDecl>,
    funcDecl: Vec<FuncDecl>,
    funcBody: FuncBody,
}

impl Comp {
    pub fn new(tc: &mut TokenCollection) -> Self {
        let mut varDecl = vec!();
        let mut funcDecl = vec!();
        let mut funcBody;

        while let Some(next_token) = tc.peek_next_token_type() {
            match next_token {
                TokenType::Comment => {
                    tc.get_next_token();
                },
                notComment => {
                    break;
                }
            }
        }

        match tc.get_next_token().expect("Computation error").get_type() {
            TokenType::Computation => {
                //program does in fact start with main.
                //dont really need to do anything with that
            },
            err => {
                //How in the world did you not get a main token??
                // Compiler Error :
                panic!("Expecting file to start with keyword 'main', found unexpected Token: {:?}", err);
            },
        }

        //Start by getting next token.
        while let Some(next_token) = tc.peek_next_token_type() {
            match next_token {
                TokenType::Var | TokenType::Array => {
                    //found variable declaration
                    varDecl.push(VarDecl::new(tc));
                },
                TokenType::FuncDecl => {
                    //no variable declaration found, but Function delcaration found
                    //drop through
                    break;
                },
                TokenType::LeftBrace => {
                    //no declarations found
                    //drop through
                    break;
                },
                err => {
                    // Compiler Error :
                    panic!("Expected to find VarDecl, FuncDecl, or Main body start, but found unexpected Token: {:?}", err);
                },
            }
        }

        while let Some(next_token) = tc.peek_next_token_type() {
            match next_token {
                TokenType::FuncDecl => {
                    //found funcDecl
                    funcDecl.push(FuncDecl::new(tc));
                },
                TokenType::LeftBrace => {
                    //no funcDecl found
                    break;
                },
                err => {
                    // Compiler Error :
                    panic!("Expected FuncDecl or start of Main body, but found unexpected Token: {:?}", err);
                },
            }
        }

        match tc.peek_next_token_type() {
            Some(TokenType::LeftBrace) => {
                //found body start
                tc.get_next_token();

                funcBody = FuncBody::new(tc);

                //look for closing bracket
                match tc.peek_next_token_type() {
                    Some(TokenType::RightBrace) => {
                        tc.get_next_token();
                    },
                    None => {
                        // Compiler Error :
                        panic!("Expected '}}' Token at end of main body, found EOF.");
                    },
                    err => {
                        // Compiler Error :
                        panic!("Expected '}}' Token at end of main body, found unexpected Token: {:?}", err);
                    },
                }
            },
            None => {
                // Compiler Error :
                panic!("Expected start to main body, found EOF.");
            },
            err => {
                // Compiler Error :
                panic!("Expected '{{' Token to indicate body start, found unexpected Token: {:?}", err);
            },
        }

        match tc.peek_next_token_type() {
            Some(TokenType::ComputationEnd) => {
                //found end of main computation
                tc.get_next_token();    //consume '.', return
            },
            None => {
                // Compiler Error :
                panic!("Expected end of main body Token '.', found EOF.");
            },
            err => {
                // Compiler Error :
                panic!("Expected end of main body Token '.', found unexpected Token: {:?}", err);
            },
        }

        Comp { node_type: TokenType::Computation, varDecl, funcDecl, funcBody }
    }

    pub fn get_value(&self) -> (Vec<VarDecl>, Vec<FuncDecl>, FuncBody)  {
        return (self.varDecl.to_vec(), self.funcDecl.to_vec(), self.funcBody.clone())
    }

    pub fn get_type(&self) -> TokenType {
        self.node_type.clone()
    }

    pub fn to_ir(self) {
        // TODO : All of this.
        let mut graph : Graph<Node, i32> = Graph::new();
        let mut irManager = IRManager::new();

        let initial_node = Node::new(&mut irManager);

        for var in self.varDecl {
            // These are the global variable declarations.
            // Build the variable tracker here, and give unique tags.
            var.to_ir(graph, initial_node, irManager);
        }
    }
}