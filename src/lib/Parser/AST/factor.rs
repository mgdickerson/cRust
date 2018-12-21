use lib::Lexer::token::TokenCollection;
use lib::Lexer::token::TokenType;
use Parser::AST::designator::Designator;
use Parser::AST::number::Number;
//use Parser::AST::func_call::FuncCall;
use Parser::AST::expression::Expression;

pub struct Factor {
    node_type: TokenType,
    design: Option<Designator>,
    number: Option<Number>,
    //func_Call: Option<FuncCall>,
    expression: Option<Expression>,
}

impl Factor {
    pub fn new(tc: &mut TokenCollection) {
        let

        match tc.peek_next_token_type() {
            Some(TokenType::Ident) => {
                designator(tc);
            },
            Some(TokenType::Number) => {
                number(tc);
            },
            Some(TokenType::FuncCall) => {
                func_call(tc);
            },
            Some(TokenType::LeftBrace) => {
                //consume token, call self
                tc.get_next_token();
                expression(tc);

                //handle closing brace in initial call of brace so all braces ar self contained.
                match tc.peek_next_token_type() {
                    Some(TokenType::RightBrace) => {
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
    }
}