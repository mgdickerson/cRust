use lib::Lexer::token::TokenCollection;
use lib::Lexer::token::TokenType;
use Parser::AST::ident::Ident;

#[derive(Debug,Clone)]
pub struct FuncParam {
    node_type: TokenType,
    parameters: Vec<Ident>
}

impl FuncParam {
    pub fn new(tc: &mut TokenCollection) -> Self {
        let mut parameters = vec!();

        while let Some(next_token) = tc.peek_next_token_type() {
            match next_token {
                TokenType::Ident => {
                    //get parameter ident
                    parameters.push(Ident::new(tc));
                },
                TokenType::Comma => {
                    //consume token
                    tc.get_next_token();
                    match tc.peek_next_token_type() {
                        Some(TokenType::Ident) => {
                            //all is well, drop through
                            continue;
                        },
                        None => {
                            // Compiler Error :
                            panic!("Unexpected EOF, expected Ident Token following ',' in function param.");
                        },
                        err => {
                            // Compiler Error :
                            panic!("Expected Ident Token following ',', found unexpected Token: {:?}", err);
                        },
                    }
                },
                TokenType::RightPara => {
                    //end of function, return to func_ident but do not consume token
                    break
                },
                err => {
                    // Compiler Error :
                    panic!("Expected Function Parameters, was unable to parse: {:?}", err);
                },
            }
        }

        FuncParam { node_type: TokenType::FuncParam, parameters }
    }

    pub fn get_value(&self) -> Vec<Ident>  {
        self.parameters.to_vec()
    }

    pub fn get_type(&self) -> TokenType {
        self.node_type.clone()
    }
}