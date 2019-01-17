use lib::Lexer::token::Token;
use lib::Lexer::token::TokenType;
use lib::Lexer::token::TokenCollection;
use Parser::AST::term::Term;

use super::{Node, NodeId, NodeData, IRManager, Value, ValTy, Op, InstTy};
use super::Graph;

#[derive(Debug,Clone)]
pub enum ExpList {
    term(Term),
    math_op(Token),
}

#[derive(Debug,Clone)]
pub struct Expression {
    node_type: TokenType,
    exp_list: Vec<ExpList>,
}

impl Expression {
    pub fn new(tc: &mut TokenCollection) -> Self {
        let mut exp_list = Vec::new();

        exp_list.push(ExpList::term(Term::new(tc)));

        loop {
            //handle MathOp possibility
            match tc.peek_next_token_type() {
                Some(TokenType::AddOp) | Some(TokenType::SubOp) => {
                    //MathOp found, call another term.

                    exp_list.push(ExpList::math_op(tc.get_next_token().unwrap()));
                    exp_list.push(ExpList::term(Term::new(tc)));
                },
                None => {
                    // Compiler Error :
                    panic!("Unexpected EOF in expression.");
                },
                _ => {
                    //If there is no MathOp, return. Dont do any other debugging or logic here.
                    return Expression { node_type: TokenType::Expression, exp_list }
                },
            }
        }
    }

    pub fn get_value(&self) -> Vec<ExpList>  {
        return self.exp_list.clone()
    }

    pub fn get_type(&self) -> TokenType {
        self.node_type.clone()
    }

    pub fn to_ir(self, graph: &mut Graph<Node, i32>, current_node: &mut Node, irm: &mut IRManager) -> Option<Value> {
        let mut previous_expr = None;
        let mut current_math_op = None;

        for expr in self.exp_list {
            match expr {
                ExpList::term(term) => {
                    match current_math_op {
                        Some(TokenType::AddOp) => {
                            let current_expr = term.to_ir(graph,current_node,irm).expect("Expected Valid Value, found None.");
                            let inst = irm.build_op_x_y(previous_expr.unwrap(), current_expr, InstTy::add);

                            current_node.get_mut_data_ref().add_instruction(inst.clone());
                            previous_expr = Some(Value::new(ValTy::op(inst)));
                        },
                        Some(TokenType::SubOp) => {
                            let current_expr = term.to_ir(graph,current_node,irm).expect("Expected Valid Value, found None.");
                            let inst = irm.build_op_x_y(previous_expr.unwrap(), current_expr, InstTy::sub);

                            current_node.get_mut_data_ref().add_instruction(inst.clone());
                            previous_expr = Some(Value::new(ValTy::op(inst)));
                        },
                        None => {
                            previous_expr = term.to_ir(graph,current_node,irm);
                        },
                        _ => { panic!("Expected Math Op + or - (or none) but some other was found."); }
                    }
                },
                ExpList::math_op(math_op) => {
                    current_math_op = Some(math_op.get_type());
                },
            }
        }

        previous_expr
    }

}