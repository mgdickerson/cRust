use lib::Lexer::token::Token;

pub struct Number {
    // I think it is just integers.
    // TODO : Check if just int.
    token: Token,
    value: i64,
    debugLine: String,
}

impl Number {
    pub fn new(token: Token) -> Self {
        // TODO : How do i do this again?
<<<<<<< HEAD
        Number{ value: token.get_contents().parse::<i64>().unwrap(),
=======
        Number{ value: token.clone().get_contents().parse::<i64>().unwrap(),
>>>>>>> develop
                 token: token,
                 // this will be awesome later, promise
                 // probably want to do some cool combo
                 // where i include both line, line #,
                 // and point out specific spot in the line
                 debugLine: String::from("test") }
    }

    pub fn get_value(self) -> i64 {
        self.value.clone()
    }

    pub fn get_debug(self) -> String {
        self.debugLine.clone()
    }
}