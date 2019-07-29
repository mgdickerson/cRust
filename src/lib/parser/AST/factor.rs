use lib::Lexer::token::TokenCollection;
use lib::Lexer::token::TokenType;
use Parser::AST::designator::Designator;
use Parser::AST::expression::Expression;
use Parser::AST::func_call::FuncCall;
use Parser::AST::number::Number;

use super::Graph;
use super::{IRGraphManager, InstTy, Node, NodeData, NodeId, Op, ValTy, Value};
use super::{Rc, RefCell};
use lib::Graph::graph_manager::GraphManager;

#[derive(Debug, Clone)]
pub enum FactorType {
    desig(Designator),
    num(Number),
    func_call(FuncCall),
    expr(Expression),
}

#[derive(Debug, Clone)]
pub struct Factor {
    node_type: TokenType,
    factor: Option<FactorType>,
}

impl Factor {
    pub fn new(tc: &mut TokenCollection) -> Self {
        let mut factor = None;
        let node_type = TokenType::None;

        match tc.peek_next_token_type() {
            Some(TokenType::Ident) => {
                factor = Some(FactorType::desig(Designator::new(tc)));
            }
            Some(TokenType::Number) => {
                factor = Some(FactorType::num(Number::new(tc)));
            }
            Some(TokenType::FuncCall) => {
                factor = Some(FactorType::func_call(FuncCall::new(tc)));
            }
            Some(TokenType::LeftPara) => {
                //consume token, call self
                tc.get_next_token();
                factor = Some(FactorType::expr(Expression::new(tc)));

                //handle closing brace in initial call of brace so all braces ar self contained.
                match tc.peek_next_token_type() {
                    Some(TokenType::RightPara) => {
                        tc.get_next_token();
                        //fall through
                    }
                    None => {
                        // Compiler Error :
                        panic!("Expected Closing ')' Token for expression, found EOF.");
                    }
                    err => {
                        // Compiler Error :
                        panic!("Expected Closing ')' Token for expression, found unexpected Token: {:?}", err);
                    }
                }
            }
            None => {
                // Compiler Error :
                panic!("Expected Designator, Number, Function call, or '(' Token, found EOF.");
            }
            err => {
                // Compiler Error :
                panic!("Expected Designator, Number, Function Call, or '(' Token, found unexpected {:?}", err);
            }
        }

        Factor { node_type, factor }
    }

    pub fn get_value(&self) -> FactorType {
        return self.factor.clone().unwrap();
    }

    pub fn get_type(&self) -> TokenType {
        self.node_type.clone()
    }

    pub fn to_ir(self, irgm: &mut IRGraphManager) -> Option<Value> {
        match self.factor {
            Some(FactorType::desig(desig)) => {
                let (result, expr_array) = desig.get_value();

                if expr_array.is_empty() {
                    return Some(Value::new(ValTy::var(
                        irgm.get_current_unique(&result.get_value()),
                    )));
                }

                let val_array = expr_array
                    .iter()
                    .filter_map(|expr| expr.to_owned().to_ir(irgm))
                    .collect::<Vec<Value>>();

                let uniq_arr = irgm
                    .array_manager()
                    .get_array_ref(result.get_value())
                    .clone();
                let ret_val = irgm.build_array_inst(uniq_arr, val_array, None);

                return Some(ret_val);
            }
            Some(FactorType::num(num)) => {
                let result = num.get_value();
                return Some(Value::new(ValTy::con(result)));
            }
            Some(FactorType::func_call(func)) => {
                return func.to_ir(irgm);
            }
            Some(FactorType::expr(expr)) => {
                return expr.to_ir(irgm);
            }
            None => panic!(),
        }

        // This should be an error as it should never reach this point.
        // Though currently func_call will fall through to this.
        None
    }

    pub fn scan_globals(&self, irgm: &mut IRGraphManager) {
        match &self.factor {
            Some(FactorType::desig(desig)) => {
                desig.scan_globals(irgm);
            }
            Some(FactorType::expr(expr)) => {
                expr.scan_globals(irgm);
            }
            Some(FactorType::func_call(func)) => {
                func.scan_globals(irgm);
            }
            _ => {
                // nothing else would produce a variable
            }
        }
    }
}
