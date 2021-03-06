use lib::Lexer::token::TokenCollection;
use lib::Lexer::token::TokenType;

use Parser::AST::func_body::FuncBody;
use Parser::AST::func_decl::FuncDecl;
use Parser::AST::var_decl::VarDecl;

use super::Graph;
use super::{IRGraphManager, InstTy, Node, NodeData, NodeId, NodeType, Op, ValTy, Value};
use lib::Graph::graph_manager::GraphManager;
use lib::Graph::node::NodeType::exit;
use lib::Utility::display;

#[derive(Debug, Clone)]
pub struct Comp {
    node_type: TokenType,
    varDecl: Vec<VarDecl>,
    funcDecl: Vec<FuncDecl>,
    funcBody: FuncBody,
}

impl Comp {
    pub fn new(tc: &mut TokenCollection) -> Self {
        let mut varDecl = vec![];
        let mut funcDecl = vec![];
        let funcBody;

        while let Some(next_token) = tc.peek_next_token_type() {
            match next_token {
                TokenType::Comment => {
                    tc.get_next_token();
                }
                _ => {
                    break;
                }
            }
        }

        match tc.get_next_token().expect("Computation error").get_type() {
            TokenType::Computation => {
                //program does in fact start with main.
                //dont really need to do anything with that
            }
            err => {
                //How in the world did you not get a main token??
                // Compiler Error :
                panic!(
                    "Expecting file to start with keyword 'main', found unexpected Token: {:?}",
                    err
                );
            }
        }

        //Start by getting next token.
        while let Some(next_token) = tc.peek_next_token_type() {
            match next_token {
                TokenType::Var | TokenType::Array => {
                    //found variable declaration
                    varDecl.push(VarDecl::new(tc));
                }
                TokenType::FuncDecl => {
                    //no variable declaration found, but Function delcaration found
                    //drop through
                    break;
                }
                TokenType::LeftBrace => {
                    //no declarations found
                    //drop through
                    break;
                }
                err => {
                    // Compiler Error :
                    panic!("Expected to find VarDecl, FuncDecl, or Main body start, but found unexpected Token: {:?}", err);
                }
            }
        }

        while let Some(next_token) = tc.peek_next_token_type() {
            match next_token {
                TokenType::FuncDecl => {
                    //found funcDecl
                    funcDecl.push(FuncDecl::new(tc));
                }
                TokenType::LeftBrace => {
                    //no funcDecl found
                    break;
                }
                err => {
                    // Compiler Error :
                    panic!(
                        "Expected FuncDecl or start of Main body, but found unexpected Token: {:?}",
                        err
                    );
                }
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
                    }
                    None => {
                        // Compiler Error :
                        panic!("Expected '}}' Token at end of main body, found EOF.");
                    }
                    err => {
                        // Compiler Error :
                        panic!(
                            "Expected '}}' Token at end of main body, found unexpected Token: {:?}",
                            err
                        );
                    }
                }
            }
            None => {
                // Compiler Error :
                panic!("Expected start to main body, found EOF.");
            }
            err => {
                // Compiler Error :
                panic!(
                    "Expected '{{' Token to indicate body start, found unexpected Token: {:?}",
                    err
                );
            }
        }

        match tc.peek_next_token_type() {
            Some(TokenType::ComputationEnd) => {
                //found end of main computation
                tc.get_next_token(); //consume '.', return
            }
            None => {
                // Compiler Error :
                panic!("Expected end of main body Token '.', found EOF.");
            }
            err => {
                // Compiler Error :
                panic!("Expected end of// fall through, as I cant access var_counter main body Token '.', found unexpected Token: {:?}", err);
            }
        }

        Comp {
            node_type: TokenType::Computation,
            varDecl,
            funcDecl,
            funcBody,
        }
    }

    pub fn get_value(&self) -> (Vec<VarDecl>, Vec<FuncDecl>, FuncBody) {
        return (
            self.varDecl.to_vec(),
            self.funcDecl.to_vec(),
            self.funcBody.clone(),
        );
    }

    pub fn get_type(&self) -> TokenType {
        self.node_type.clone()
    }

    pub fn to_ir(self) -> IRGraphManager {
        // This graph is directed.
        let mut ir_graph_manager = IRGraphManager::new();

        for var in self.varDecl {
            // These are the global variable declarations.
            // Build the variable tracker here, and give unique tags.
            var.to_ir(&mut ir_graph_manager, true, None);
        }

        for func in self.funcDecl {
            func.to_ir(&mut ir_graph_manager);
        }

        ir_graph_manager.graph_manager().set_main_node();
        self.funcBody.to_ir(&mut ir_graph_manager);

        let ret_0 = ir_graph_manager.build_op_x(Value::new(ValTy::con(0)), InstTy::ret);
        ir_graph_manager.graph_manager().add_instruction(ret_0);

        let bottom_node = ir_graph_manager
            .graph_manager()
            .get_mut_ref_current_node_index()
            .clone();
        let exit_index = ir_graph_manager
            .new_node(String::from("Exit"), NodeType::exit)
            .clone();

        ir_graph_manager
            .graph_manager()
            .add_edge(bottom_node, exit_index);

        //println!("{:?}", ir_graph_manager.variable_manager().clone().get_var_map());
        //graph_manager.add_current_node_to_graph();
        //let clone = ir_graph_manager.variable_manager();
        //println!("{:?}", clone);

        ir_graph_manager
        /*
        println!("{:?}", irgm.get_var_manager().get_var_map());
        let (nodes, edges) = graph_manager.get_graph().into_nodes_edges();
        for node in nodes {
            for op in node.weight.get_data().get() {
                println!("{}", op.get_inst_block());
            }
        }
        */
    }
}
