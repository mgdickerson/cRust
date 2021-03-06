use lib::Lexer::token::Token;
use lib::Lexer::token::TokenCollection;
use lib::Lexer::token::TokenType;
use Parser::AST::factor::{Factor, FactorType};

use super::Graph;
use super::{IRGraphManager, InstTy, Node, NodeData, NodeId, Op, ValTy, Value};
use super::{Rc, RefCell};
use lib::Graph::graph_manager::GraphManager;

#[derive(Debug, Clone)]
enum TermList {
    factor(Factor),
    operation(Token),
}

#[derive(Debug, Clone)]
pub struct Term {
    node_type: TokenType,
    term_list: Vec<TermList>,
}

impl Term {
    pub fn new(tc: &mut TokenCollection) -> Self {
        let mut term_list = Vec::new();

        term_list.push(TermList::factor(Factor::new(tc)));

        loop {
            //handle MulOp possibility
            match tc.peek_next_token_type() {
                Some(TokenType::MulOp) | Some(TokenType::DivOp) => {
                    //MulOp found, consume then call factor again
                    term_list.push(TermList::operation(tc.get_next_token().unwrap()));
                    term_list.push(TermList::factor(Factor::new(tc)));
                }
                None => {
                    // Compiler Error :
                    panic!("Unexpected EOF in term.");
                }
                _ => {
                    //If no MulOp, return
                    return Term {
                        node_type: TokenType::Term,
                        term_list,
                    };
                }
            }
        }
    }

    pub fn get_value(&self) -> Vec<TermList> {
        return self.term_list.clone();
    }

    pub fn get_type(&self) -> TokenType {
        self.node_type.clone()
    }

    pub fn to_ir(self, irgm: &mut IRGraphManager) -> Option<Value> {
        let mut previous_term: Option<Value> = None;
        let mut current_math_op = None;

        for term in self.term_list {
            match term {
                TermList::factor(factor) => {
                    match current_math_op {
                        Some(TokenType::MulOp) => {
                            let current_term = factor
                                .to_ir(irgm)
                                .expect("Expected Valid Value, found None.");

                            // Would be nice to at least make all instructions at least 1 constant at most.
                            if let ValTy::con(prev_con) =
                                previous_term.clone().unwrap().get_value().clone()
                            {
                                let const_split = irgm.build_op_x_y(
                                    Value::new(ValTy::con(0)),
                                    Value::new(ValTy::con(prev_con)),
                                    InstTy::add,
                                );
                                previous_term =
                                    Some(irgm.graph_manager().add_instruction(const_split));
                            }

                            let inst = irgm.build_op_x_y(
                                previous_term.unwrap(),
                                current_term,
                                InstTy::mul,
                            );

                            let inst_val = irgm.graph_manager().add_instruction(inst);
                            previous_term = Some(inst_val);
                        }
                        Some(TokenType::DivOp) => {
                            let current_term = factor
                                .to_ir(irgm)
                                .expect("Expected Valid Value, found None.");

                            // Would be nice to at least make all instructions at least 1 constant at most.
                            if let ValTy::con(prev_con) =
                                previous_term.clone().unwrap().get_value().clone()
                            {
                                let const_split = irgm.build_op_x_y(
                                    Value::new(ValTy::con(0)),
                                    Value::new(ValTy::con(prev_con)),
                                    InstTy::add,
                                );
                                previous_term =
                                    Some(irgm.graph_manager().add_instruction(const_split));
                            }

                            let inst = irgm.build_op_x_y(
                                previous_term.unwrap(),
                                current_term,
                                InstTy::div,
                            );

                            let inst_val = irgm.graph_manager().add_instruction(inst);
                            previous_term = Some(inst_val);
                        }
                        None => {
                            previous_term = factor.to_ir(irgm);
                        }
                        _ => {
                            panic!("Found math_op in term that was not * or /");
                        }
                    }
                }
                TermList::operation(math_op) => {
                    current_math_op = Some(math_op.get_type());
                }
            }
        }

        previous_term
    }

    pub fn scan_globals(&self, irgm: &mut IRGraphManager) {
        for term in &self.term_list {
            match term {
                TermList::factor(factor) => {
                    factor.scan_globals(irgm);
                }
                _ => {
                    // Does not produce a global
                }
            }
        }
    }
}
