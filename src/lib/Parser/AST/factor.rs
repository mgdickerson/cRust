use lib::Lexer::token::TokenCollection;
use lib::Lexer::token::TokenType;
use Parser::AST::designator::Designator;
use Parser::AST::number::Number;
use Parser::AST::func_call::FuncCall;
use Parser::AST::expression::Expression;

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
}