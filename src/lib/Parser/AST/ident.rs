use lib::Lexer::token::Token;

pub struct Ident {
    token: Token,
    value: String,
    debugLine: String,
}

<<<<<<< HEAD
impl Ident {
    pub fn new(token: Token) -> Self {
        //Ident{ token: token,
        //    value: token.get_contents(),
        //    debugLine: String::from("test")}
=======
//random test

impl Ident {
    pub fn new(token: Token) -> Self {
        Ident{ token: token.clone(),
            value: token.get_contents(),
            debugLine: String::from("test")}
>>>>>>> develop
    }

    pub fn get_value(self) -> String {
        self.value.clone()
    }

    pub fn get_debug(self) -> String {
        self.debugLine.clone()
    }
}