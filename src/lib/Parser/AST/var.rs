use lib::Lexer::token::TokenCollection;
use lib::Lexer::token::TokenType;
use Parser::AST;

pub struct Var {
    var_type: TokenType,
    var_vec: Vec<AST::ident::Ident>,
    debugLine: String,
}

impl Var {
    pub fn new(tc: &mut TokenCollection) -> Self {
        let mut varList : Vec<AST::ident::Ident> = vec!();
        let mut varTokenType = TokenType::None;

        match tc.get_next_token().expect("Var Error").get_type() {
            TokenType::Var => {
                //This is accepted behavior, pass through.
                varTokenType = TokenType::Var;
            },
            err => {
                // Compiler Error : 
                panic!("Expected Variable declaration, found unexpected Token: {:?}", err);
            }
        }

        while let Some(next_token) = tc.peek_next_token_type() {
            match next_token {
                TokenType::Ident => {
                    varList.push(AST::ident::Ident::new(tc));
                },
                TokenType::Comma => {
                    //consume comma token
                    tc.get_next_token();
                },
                TokenType::SemiTermination => {
                    //consume semicolon and return. 
                    tc.get_next_token();
                    break;
                },
                err => {
                    // Compiler Error : 
                    panic!("Unable to parse token in variable declaration: {:?}", err);
                },
            }
        }

        Var{ var_type: varTokenType, var_vec: varList, debugLine: String::from("test")}
    }

    pub fn get_value(&self) -> Vec<AST::ident::Ident> {
        self.var_vec.to_vec()
    }

    pub fn get_type(&self) -> TokenType {
        self.var_type.clone()
    }

    pub fn get_debug(self) -> String {
        self.debugLine.clone()
    }
}