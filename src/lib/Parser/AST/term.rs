use lib::Lexer::token::TokenCollection;
use lib::Lexer::token::TokenType;
use lib::Lexer::token::Token;
use Parser::AST::factor::{Factor,FactorType};

use super::{Node, NodeId, NodeData, IRManager, Value, ValTy, Op, InstTy};
use super::Graph;

#[derive(Debug,Clone)]
enum TermList {
    factor(Factor),
    operation(Token),
}

#[derive(Debug,Clone)]
pub struct Term {
    node_type: TokenType,
    term_list: Vec<TermList>,
}

impl Term {
    pub fn new(tc: &mut TokenCollection) -> Self {
        let mut term_list = Vec::new();

        let mut factors : Vec<Factor> = vec!();
        let mut operations : Vec<Token> = vec!();

        term_list.push(TermList::factor(Factor::new(tc)));

        loop {
            //handle MulOp possibility
            match tc.peek_next_token_type() {
                Some(TokenType::MulOp) | Some(TokenType::DivOp) => {
                    //MulOp found, consume then call factor again
                    term_list.push(TermList::operation(tc.get_next_token().unwrap()));
                    term_list.push(TermList::factor(Factor::new(tc)));
                },
                None => {
                    // Compiler Error :
                    panic!("Unexpected EOF in term.");
                },
                _ => {
                    //If no MulOp, return
                    return Term { node_type: TokenType::Term, term_list }
                },
            }
        }
    }

    pub fn get_value(&self) -> Vec<TermList>  {
        return self.term_list.clone()
    }

    pub fn get_type(&self) -> TokenType {
        self.node_type.clone()
    }

    pub fn to_ir(self, graph: &mut Graph<Node, i32>, current_node: &mut Node, irm: &mut IRManager) -> Option<Value> {
        let mut previous_term = None;
        let mut current_math_op = None;

        for term in self.term_list {
            match term {
                TermList::factor(factor) => {
                    match current_math_op {
                        Some(TokenType::MulOp) => {
                            let current_term = factor.to_ir(graph,current_node,irm).expect("Expected Valid Value, found None.");
                            let inst = irm.build_op_x_y(previous_term.unwrap(), current_term, InstTy::mul);

                            current_node.get_mut_data_ref().add_instruction(inst.clone());
                            previous_term = Some(Value::new(ValTy::op(inst)));
                        },
                        Some(TokenType::DivOp) => {
                            let current_term = factor.to_ir(graph,current_node,irm).expect("Expected Valid Value, found None.");
                            let inst = irm.build_op_x_y(previous_term.unwrap(), current_term, InstTy::div);

                            current_node.get_mut_data_ref().add_instruction(inst.clone());
                            previous_term = Some(Value::new(ValTy::op(inst)));
                        },
                        None => {
                            previous_term = factor.to_ir(graph,current_node,irm);
                        },
                        _ => { panic!("Found math_op in term that was not * or /"); }
                    }
                },
                TermList::operation(math_op) => {
                    current_math_op = Some(math_op.get_type());
                },
            }
        }

        previous_term
    }

}