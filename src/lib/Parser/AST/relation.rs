use lib::Lexer::token::Token;
use lib::Lexer::token::TokenCollection;
use lib::Lexer::token::TokenType;
use Parser::AST::expression::Expression;

use super::Graph;
use super::{IRGraphManager, InstTy, Node, NodeData, NodeId, Op, ValTy, Value};
use super::{Rc, RefCell};
use lib::Graph::graph_manager::GraphManager;

#[derive(Debug, Clone)]
pub struct Relation {
    node_type: TokenType,
    leftExp: Expression,
    relOp: Token,
    rightExp: Expression,
}

impl Relation {
    pub fn new(tc: &mut TokenCollection) -> Self {
        let leftExp = Expression::new(tc);
        let mut relOp = Token::new(TokenType::None, String::from(""));

        match tc.peek_next_token_type() {
            Some(TokenType::RelOp) => {
                //consume token
                relOp = tc.get_next_token().unwrap();
            }
            None => {
                // Compiler Error :
                panic!("Expected RelOp token, found EOF.");
            }
            err => {
                // Compiler Error :
                panic!(
                    "Expected RelOp token, unexpected Token {:?} was found instead.",
                    err
                );
            }
        }

        let rightExp = Expression::new(tc);

        //relation is built, return
        Relation {
            node_type: TokenType::Relation,
            leftExp,
            relOp,
            rightExp,
        }
    }

    pub fn get_value(&self) -> (Expression, Token, Expression) {
        return (
            self.leftExp.clone(),
            self.relOp.clone(),
            self.rightExp.clone(),
        );
    }

    pub fn get_type(&self) -> TokenType {
        self.node_type.clone()
    }

    pub fn to_ir(self, irgm: &mut IRGraphManager, branch_location: Value) -> Value {
        let mut leftCompVal = self
            .leftExp
            .to_ir(irgm)
            .expect("Expected Left Comp Op, none found");
        let mut rightCompVal = self
            .rightExp
            .to_ir(irgm)
            .expect("Expected Right Comp Op, none found");

        // Technically only the left value cant be a const, as the right value can be const with cmpi
        if let ValTy::con(left_con) = leftCompVal.get_value().clone() {
            let const_split = irgm.build_op_x_y(
                Value::new(ValTy::con(0)),
                Value::new(ValTy::con(left_con)),
                InstTy::add,
            );
            leftCompVal = irgm.graph_manager().add_instruction(const_split);
        }

        /*if let ValTy::con(right_con) = rightCompVal.get_value().clone() {
            let const_split = irgm.build_op_x_y(Value::new(ValTy::con(0)), Value::new(ValTy::con(right_con)), InstTy::add);
            rightCompVal = irgm.graph_manager().add_instruction(const_split);
        }*/

        let inst = irgm.build_op_x_y(leftCompVal, rightCompVal, InstTy::cmp);
        let inst_val = irgm.graph_manager().add_instruction(inst);

        match self.relOp.get_contents().as_ref() {
            "==" => {
                let rel_inst = irgm.build_op_x_y(inst_val.clone(), branch_location, InstTy::bne);
                irgm.graph_manager().add_instruction(rel_inst);
            }
            "!=" => {
                let rel_inst = irgm.build_op_x_y(inst_val.clone(), branch_location, InstTy::beq);
                irgm.graph_manager().add_instruction(rel_inst);
            }
            "<" => {
                let rel_inst = irgm.build_op_x_y(inst_val.clone(), branch_location, InstTy::bge);
                irgm.graph_manager().add_instruction(rel_inst);
            }
            "<=" => {
                let rel_inst = irgm.build_op_x_y(inst_val.clone(), branch_location, InstTy::bgt);
                irgm.graph_manager().add_instruction(rel_inst);
            }
            ">" => {
                let rel_inst = irgm.build_op_x_y(inst_val.clone(), branch_location, InstTy::ble);
                irgm.graph_manager().add_instruction(rel_inst);
            }
            ">=" => {
                let rel_inst = irgm.build_op_x_y(inst_val.clone(), branch_location, InstTy::blt);
                irgm.graph_manager().add_instruction(rel_inst);
            }
            _ => {
                panic!("Error: Expected a relOp token, but was not found.");
            }
        }

        inst_val
    }

    pub fn scan_globals(&self, irgm: &mut IRGraphManager) {
        self.leftExp.scan_globals(irgm);
        self.rightExp.scan_globals(irgm);
    }
}
