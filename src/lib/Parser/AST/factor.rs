use lib::Lexer::token::TokenCollection;
use lib::Lexer::token::TokenType;
use Parser::AST::designator::Designator;
use Parser::AST::number::Number;
use Parser::AST::func_call::FuncCall;
use Parser::AST::expression::Expression;

use super::{Node, NodeId, NodeData, IRGraphManager, Value, ValTy, Op, InstTy};
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
        let node_type = TokenType::None;

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

    pub fn to_ir(self, irgm : &mut IRGraphManager) -> Option<Value> {
        match self.factor {
            Some(FactorType::desig(desig)) => {
                let (result, expr_array) = desig.get_value();

                if expr_array.is_empty() {
                    let block_num = irgm.get_block_num();
                    let inst_num = irgm.get_inst_num() + 1;
                    return Some(Value::new(
                        ValTy::var(irgm.get_current_unique(result.get_value()).clone())));
                }

                let val_array = expr_array.iter()
                    .filter_map(|expr| {
                        expr.to_owned().to_ir(irgm)
                    }).collect::<Vec<Value>>();

                let uniq_arr = irgm.array_manager().get_array_ref(result.get_value()).clone();
                let inst_list = irgm.build_array_inst(uniq_arr, val_array, None);

                let ret_val = Value::new(ValTy::op(inst_list.last().expect("There should be a final Op.").clone()));

                for inst in inst_list {
                    irgm.graph_manager().add_instruction(inst);
                }

                return Some(ret_val);
            },
            Some(FactorType::num(num)) => {
                let result = num.get_value();
                return Some(Value::new(ValTy::con(result)));
            },
            Some(FactorType::func_call(func)) => {
                // TODO : This is a rough impl, just to get the "call" to print out.
                // TODO : Still needs to be implemented.
                let inst = irgm.build_spec_op(Vec::new(), InstTy::call);
                irgm.graph_manager().add_instruction(inst.clone());
                return Some(Value::new(ValTy::op(inst)));
            },
            Some(FactorType::expr(expr)) => {
                return expr.to_ir(irgm);
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