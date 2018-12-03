use lib::Lexer::token::TokenCollection;
use lib::Lexer::token::TokenType;
use Parser::AST;

pub struct Var {
    variables: Vec<AST::ident::Ident>,
    debugLine: String,
}

impl Var {
    pub fn new(tc: &mut TokenCollection) -> Self {
        let mut varList : Vec<AST::ident::Ident> = vec!();
        
        match tc.get_next_token().expect("Var Error").get_type() {
            TokenType::Var => {
                //This is accepted behavior, pass through. 
            },
            err => {
                // Compiler Error : 
                panic!("Expected Variable declaration, found unexpected Token: {:?}", err);
            }
        }

        while let Some(next_token) = tc.peek_next_token_type() {
            match next_token {
                TokenType::Ident => {
                    varList.push(AST::ident::Ident::new(tc.get_next_token().unwrap()));
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

        Var{ variables: varList, debugLine: String::from("test")}
    }
}