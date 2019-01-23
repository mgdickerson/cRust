use lib::Lexer::token::TokenCollection;
use lib::Lexer::token::TokenType;
use Parser::AST::designator::Designator;
use Parser::AST::number::Number;
use Parser::AST::func_call::FuncCall;
use Parser::AST::expression::Expression;

use super::{Node, NodeId, NodeData, IRManager, Value, ValTy, Op, InstTy};
use super::Graph;
use lib::Graph::graph_manager::GraphManager;

#[derive(Debug,Clone)]
pub enum FactorType {
    desig(Designator),
    num(Number),
    func_call(FuncCall),
    expr(Expression),
}

#[derive(Debug,Clone)]
pub struct Factor {
    node_type: TokenType,
    factor: Option<FactorType>,
}

impl Factor {
    pub fn new(tc: &mut TokenCollection) -> Self {
        let mut factor = None;
        let mut node_type = TokenType::None;

        match tc.peek_next_token_type() {
            Some(TokenType::Ident) => {
                factor = Some(FactorType::desig(Designator::new(tc)));
            },
            Some(TokenType::Number) => {
                factor = Some(FactorType::num(Number::new(tc)));
            },
            Some(TokenType::FuncCall) => {
                factor = Some(FactorType::func_call(FuncCall::new(tc)));
            },
            Some(TokenType::LeftPara) => {
                //consume token, call self
                tc.get_next_token();
                factor = Some(FactorType::expr(Expression::new(tc)));

                //handle closing brace in initial call of brace so all braces ar self contained.
                match tc.peek_next_token_type() {
                    Some(TokenType::RightPara) => {
                        tc.get_next_token();
                        //fall through
                    },
                    None => {
                        // Compiler Error :
                        panic!("Expected Closing ')' Token for expression, found EOF.");
                    },
                    err => {
                        // Compiler Error :
                        panic!("Expected Closing ')' Token for expression, found unexpected Token: {:?}", err);
                    },
                }
            },
            None => {
                // Compiler Error :
                panic!("Expected Designator, Number, Function call, or '(' Token, found EOF.");
            },
            err => {
                // Compiler Error :
                panic!("Expected Designator, Number, Function Call, or '(' Token, found unexpected {:?}", err);
            },
        }

        Factor{ node_type, factor }
    }

    pub fn get_value(&self) -> FactorType  {
        return self.factor.clone().unwrap()
    }

    pub fn get_type(&self) -> TokenType {
        self.node_type.clone()
    }

    pub fn to_ir(self, graph_manager: &mut GraphManager, irm: &mut IRManager) -> Option<Value> {
        match self.factor {
            Some(FactorType::desig(desig)) => {
                // TODO : needs testing but there is SOMETHING in place for arrays
                let (result, array) = desig.get_value();

                if array.is_empty() {
                    return Some(irm.get_unique_variable(result.get_value()).get_value());
                }

                let mut array_result = result.get_value() + "[";
                let mut first = true;
                for element in array {
                    if !first {
                        array_result += ", ";
                    }
                    array_result += &element.to_ir(graph_manager,irm).expect("Expected valid Value").get_value().to_string();
                    first = false;
                }
                array_result += "]";

                // TODO : Replace with in class implementation of these.
                let inst = irm.build_op_y(Value::new(ValTy::arr(array_result)), InstTy::load);
                graph_manager.add_instruction(inst.clone());

                return Some(Value::new(ValTy::op(inst)));
            },
            Some(FactorType::num(num)) => {
                let result = num.get_value();
                return Some(Value::new(ValTy::con(result)));
            },
            Some(FactorType::func_call(func)) => {
                // TODO : This is a rough impl, just to get the "call" to print out.
                // TODO : Still needs to be implemented.
                let inst = irm.build_spec_op(Vec::new(), InstTy::call);
                graph_manager.add_instruction(inst.clone());
                return Some(Value::new(ValTy::op(inst)));
            },
            Some(FactorType::expr(expr)) => {
                return expr.to_ir(graph_manager,irm);
            },
            None => {
                panic!()
            }
        }

        // This should be an error as it should never reach this point.
        // Though currently func_call will fall through to this.
        None
    }

}