use lib::Lexer::token::TokenCollection;
use lib::Lexer::token::TokenType;
use Parser::AST::designator::Designator;
use Parser::AST::expression::Expression;

use super::{Node, NodeId, NodeData, IRGraphManager, Value, ValTy, Op, InstTy};
use super::Graph;
use lib::Graph::graph_manager::GraphManager;

#[derive(Debug,Clone)]
pub struct Assignment {
    node_type: TokenType,
    designator: Designator,
    expression: Expression,
}

impl Assignment {
    pub fn new(tc: &mut TokenCollection) -> Self {
        let mut designator;
        let mut expression;

        match tc.get_next_token().expect("Assignment Error").get_type() {
            TokenType::Assignment => {
                //expected assignment token found.
            },
            err => {
                // Compiler Error :
                panic!("Expected to find Assignment token, found unexpected Token: {:?}", err);
            },
        }

        match tc.peek_next_token_type() {
            Some(TokenType::Ident) => {
                designator = Designator::new(tc);
            },
            err => {
                // Compiler Error :
                panic!("Expected Designator for assignment variable, found unexpected Token: {:?}", err);
            },
        }

        match tc.get_next_token().expect("Assignment Op Error").get_type() {
            TokenType::AssignmentOp => {
                //expected assignment operator found, proceed to expression.
                expression = Expression::new(tc);
            },
            err => {
                // Compiler Error :
                panic!("Expected Assignment Operator '<-', found unexpected Token: {:?}", err);
            },
        }

        match tc.peek_next_token_type() {
            Some(TokenType::SemiTermination) => {
                //consume token, return.
                tc.get_next_token();
            },
            // All Possible Ending Sequences where ';' may not be necessary.
            Some(TokenType::FiStatement) | Some(TokenType::OdStatement) |
            Some(TokenType::RightBrace) | Some(TokenType::ElseStatement) |
            Some(TokenType::RightPara) => {
                //';' not required, return without consuming token.
            },
            None => {
                // Compiler Error :
                panic!("Expected end of assignment, found EOF.");
            },
            err => {
                // Compiler Error :
                panic!("Expected end of assignment, found unexpected Token: {:?}", err);
            },
        }

        Assignment { node_type: TokenType::Assignment, designator, expression }
    }

    pub fn get_value(&self) -> (Designator, Expression)  {
        return (self.designator.clone(), self.expression.clone())
    }

    pub fn get_type(&self) -> TokenType {
        self.node_type.clone()
    }

    pub fn to_ir(self, irgm : &mut IRGraphManager) {
        let (result, array) = self.designator.get_value();
        let ident;
        let inst;

        let expr_value = self.expression.to_ir(irgm).expect("Expected some expression with related Assignment Operation");

        if array.is_empty() {
            ident = Value::new(ValTy::var(irgm.make_unique_variable(result.get_value(), expr_value.clone()).clone()));
        } else {
            let mut array_result = result.get_value() + "[";
            let mut first = true;
            for element in array {
                if !first {
                    array_result += ", ";
                }
                array_result += &element.to_ir(irgm).expect("Expected valid Value").get_value().to_string();
                first = false;
            }
            array_result += "]";

            inst = irgm.build_op_y(Value::new(ValTy::arr(array_result)), InstTy::load);
            irgm.add_inst(inst.clone());

            ident = Value::new(ValTy::op(inst));
        }

        //let new_inst = irgm.build_op_x_y(ident,expr_value,InstTy::mov);
        //current_node.get_mut_data_ref().add_instruction(new_inst);
    }
}