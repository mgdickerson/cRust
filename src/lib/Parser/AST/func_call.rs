use lib::Lexer::token::TokenCollection;
use lib::Lexer::token::TokenType;
use Parser::AST::ident::Ident;
use Parser::AST::expression::Expression;

use super::{Node, NodeId, NodeData, IRGraphManager, Value, ValTy, Op, InstTy};
use super::Graph;

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
                                        expr => {
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

    pub fn to_ir(self, irgm: &mut IRGraphManager) {

        // TODO : Need to grab the function from function list first.

        match self.funcName.get_value().as_ref() {
            "InputNum" => {},
            "OutputNum" => {},
            "OutputNewLine" => {},
            func_name => {
                let uniq_func = irgm.function_manager().get_mut_function(&self.funcName.get_value());

                print!("Affected Globals: ");
                for global in uniq_func.list_affected_globals() {
                    print!("{}  ", global);
                }
                println!();

            },
        }
    }

    pub fn scan_globals(&self, irgm : &mut IRGraphManager) {
        let recursive_call = irgm.variable_manager().active_function().get_name();

        if self.funcName.get_value() == recursive_call {
            for expr in &self.variables {
                expr.scan_globals(irgm);
            }

            // There are no further global items this should call
            return
        }

        match self.funcName.get_value().as_ref() {
            "InputNum" => {},
            "OutputNum" => {},
            "OutputNewLine" => {},
            func_name => {
                let affected_globals = irgm.function_manager().get_mut_function(&self.funcName.get_value()).list_affected_globals();
                for global in affected_globals {
                    irgm.function_manager().get_mut_function(&self.funcName.get_value()).add_to_affected_globals(&global);
                }
            },
        }

        for expr in &self.variables {
            expr.scan_globals(irgm);
        }
    }
}