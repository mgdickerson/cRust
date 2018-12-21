use lib::Lexer::token::Token;
use lib::Lexer::token::TokenType;

#[derive(Debug,Clone)]
pub struct Number {
    number_type: TokenType,
    number_value: i64,
    debugLine: String,
}

impl Number {
    pub fn new(token: Token) -> Self {
        Number{ number_type: token.get_type(),
            number_value: token.get_contents().parse::<i64>().unwrap(),
                 // this will be awesome later, promise
                 // probably want to do some cool combo
                 // where i include both line, line #,
                 // and point out specific spot in the line
                 debugLine: String::from("test") }
    }

    pub fn get_value(&self) -> i64 {
        self.number_value.clone()
    }

    pub fn get_type(&self) -> TokenType { self.number_type.clone() }

    pub fn get_debug(self) -> String {
        self.debugLine.clone()
    }
}