use lib::Lexer::token::Token;
use lib::Lexer::token::TokenCollection;
use lib::Lexer::token::TokenType;

#[derive(Debug,Clone)]
pub struct Ident {
    ident_type: TokenType,
    ident_value: String,
    debugLine: String,
}
impl Ident {
    pub fn new(tc: &mut TokenCollection) -> Self {
        let token = tc.get_next_token().unwrap();
        Ident{ ident_type: token.get_type(),
            ident_value: token.get_contents(),
            debugLine: String::from("test")}
    }

    pub fn get_value(&self) -> String {
        self.ident_value.clone()
    }

    pub fn get_type(&self) -> TokenType {
        self.ident_type.clone()
    }

    pub fn get_debug(self) -> String {
        self.debugLine.clone()
    }
}