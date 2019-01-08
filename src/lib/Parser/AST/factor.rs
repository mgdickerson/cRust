use lib::Lexer::token::TokenCollection;
use lib::Lexer::token::TokenType;
use Parser::AST::designator::Designator;
use Parser::AST::number::Number;
use Parser::AST::func_call::FuncCall;
use Parser::AST::expression::Expression;

#[derive(Debug,Clone)]
pub struct Factor {
    node_type: TokenType,
    design: Option<Designator>,
    number: Option<Number>,
    func_Call: Option<FuncCall>,
    expression: Option<Expression>,
}

impl Factor {
    pub fn new(tc: &mut TokenCollection) -> Self {
        let mut node_type = TokenType::None;
        let mut design = Option::None;
        let mut number = Option::None;
        let mut func_Call = Option::None;
        let mut expression = Option::None;

        match tc.peek_next_token_type() {
            Some(TokenType::Ident) => {
                design = Option::Some(Designator::new(tc));
            },
            Some(TokenType::Number) => {
                number = Option::Some(Number::new(tc));
            },
            Some(TokenType::FuncCall) => {
                func_Call = Option::Some(FuncCall::new(tc))
            },
            Some(TokenType::LeftPara) => {
                //consume token, call self
                tc.get_next_token();
                expression = Some(Expression::new(tc));

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

        Factor{ node_type, design, number, func_Call, expression }
    }

    pub fn get_value(&self) -> (Option<Designator>, Option<Number>, Option<FuncCall>, Option<Expression> )  {
        return (self.design.clone(), self.number.clone(), self.func_Call.clone(), self.expression.clone())
    }

    pub fn get_type(&self) -> TokenType {
        self.node_type.clone()
    }
}