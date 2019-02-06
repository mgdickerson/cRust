use lib::Lexer::token::TokenCollection;
use lib::Lexer::token::TokenType;
use lib::Lexer::token::Token;
use Parser::AST::expression::Expression;

use super::{Node, NodeId, NodeData, IRGraphManager, Value, ValTy, Op, InstTy};
use super::Graph;
use lib::Graph::graph_manager::GraphManager;

#[derive(Debug,Clone)]
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
            },
            None => {
                // Compiler Error :
                panic!("Expected RelOp token, found EOF.");
            },
            err => {
                // Compiler Error :
                panic!("Expected RelOp token, unexpected Token {:?} was found instead.", err);
            },
        }

        let rightExp = Expression::new(tc);

        //relation is built, return
        Relation{ node_type: TokenType::Relation, leftExp, relOp, rightExp }
    }

    pub fn get_value(&self) -> (Expression, Token, Expression)  {
        return (self.leftExp.clone(), self.relOp.clone(), self.rightExp.clone())
    }

    pub fn get_type(&self) -> TokenType {
        self.node_type.clone()
    }

    pub fn to_ir(self, irgm: &mut IRGraphManager, branch_location: Value) -> Value {
        let leftCompVal = self.leftExp.to_ir(irgm).
            expect("Expected Left Comp Op, none found");
        let rightCompVal = self.rightExp.to_ir(irgm).
            expect("Expected Right Comp Op, none found");

        let inst = irgm.build_op_x_y(leftCompVal,rightCompVal,InstTy::cmp);
        irgm.graph_manager().add_instruction(inst.clone());
        let inst_val = Value::new(ValTy::op(inst));

        match self.relOp.get_contents().as_ref() {
            "==" => {
                let rel_inst = irgm.build_op_x_y(inst_val.clone(),branch_location,InstTy::bne);
                irgm.graph_manager().add_instruction(rel_inst);
            },
            "!=" => {
                let rel_inst = irgm.build_op_x_y(inst_val.clone(),branch_location,InstTy::beq);
                irgm.graph_manager().add_instruction(rel_inst);
            },
            "<" => {
                let rel_inst = irgm.build_op_x_y(inst_val.clone(),branch_location,InstTy::bge);
                irgm.graph_manager().add_instruction(rel_inst);
            },
            "<=" => {
                let rel_inst = irgm.build_op_x_y(inst_val.clone(),branch_location,InstTy::bgt);
                irgm.graph_manager().add_instruction(rel_inst);
            },
            ">" => {
                let rel_inst = irgm.build_op_x_y(inst_val.clone(),branch_location,InstTy::ble);
                irgm.graph_manager().add_instruction(rel_inst);
            },
            ">=" => {
                let rel_inst = irgm.build_op_x_y(inst_val.clone(),branch_location,InstTy::blt);
                irgm.graph_manager().add_instruction(rel_inst);
            },
            _ => {
                panic!("Error: Expected a relOp token, but was not found.");
            },
        }

        inst_val
    }
}