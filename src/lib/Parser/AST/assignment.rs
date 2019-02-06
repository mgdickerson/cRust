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
        let designator;
        let expression;

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
        let (result, expr_array) = self.designator.get_value();

        let expr_value = self.expression.to_ir(irgm).expect("Expected some expression with related Assignment Operation");

        if expr_array.is_empty() {
            let block_num = irgm.get_block_num();
            let inst_num = irgm.get_inst_num();
            irgm.variable_manager().make_unique_variable(result.get_value(), expr_value.clone(), block_num, inst_num);
        } else {
            let val_array = expr_array.iter()
                .filter_map(|expr| {
                    expr.to_owned().to_ir(irgm)
                }).collect::<Vec<Value>>();

            let uniq_arr = irgm.array_manager().get_array_ref(result.get_value()).clone();
            let inst_list = irgm.build_array_inst(uniq_arr, val_array, Some(expr_value));

            // TODO : Not sure why this is here, but it seems important.
            let ret_val = Value::new(ValTy::op(inst_list.last().expect("There should be a final Op.").clone()));

            for inst in inst_list {
                irgm.graph_manager().add_instruction(inst);
            }
        }
    }
}