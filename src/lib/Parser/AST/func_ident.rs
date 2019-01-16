use lib::Lexer::token::TokenCollection;
use lib::Lexer::token::TokenType;
use Parser::AST::ident::Ident;
use Parser::AST::func_param::FuncParam;

#[derive(Debug,Clone)]
pub struct FuncIdent {
    node_type: TokenType,
    funcName: Ident,
    funcParam: Option<FuncParam>,
}

impl FuncIdent {
    pub fn new(tc: &mut TokenCollection) -> Self {
        let mut funcName;
        let mut funcParam = Option::None;

        match tc.peek_next_token_type() {
            Some(TokenType::Ident) => {
                //function name
                funcName = Ident::new(tc);
            },
            None => {
                // Compiler Error :
                panic!("Expected Ident Token for function name, found EOF.");
            },
            err => {
                // Compiler Error :
                panic!("Expected Ident Token for function name, found unexpected Token: {:?}", err);
            },
        }

        match tc.peek_next_token_type() {
            Some(TokenType::LeftPara) => {
                //function parameter start
                tc.get_next_token();

                funcParam = Some(FuncParam::new(tc));

                match tc.peek_next_token_type() {
                    Some(TokenType::RightPara) => {
                        tc.get_next_token();
                    },
                    Some(TokenType::SemiTermination) => {
                        //pass through to return statement
                    },
                    None => {
                        // Compiler Error :
                        panic!("Expected ')' Token, found EOF.");
                    },
                    err => {
                        // Compiler Error :
                        panic!("Expected ')' Token at end of function parameters, found unexpected Token: {:?}", err);
                    },
                }
            },
            Some(TokenType::SemiTermination) => {
                //no parameters to pass, fall through
            },
            None => {
                // Compiler Error :
                panic!("Expected '(' Token, found EOF.");
            },
            err => {
                // Compiler Error :
                panic!("Expected '(' Token at end of function parameters, found unexpected Token: {:?}", err);
            },
        }

        FuncIdent { node_type: TokenType::FuncIdent, funcName, funcParam }
    }

    pub fn get_value(&self) -> (Ident, Option<FuncParam>) {
        return (self.funcName.clone(), self.funcParam.clone())
    }

    pub fn get_type(&self) -> TokenType {
        self.node_type.clone()
    }

}
