use lib::Lexer::token::TokenCollection;
use lib::Lexer::token::TokenType;
use Parser::AST::var::Var;
use Parser::AST::array::Array;

#[derive(Debug,Clone)]
pub struct VarDecl {
    node_type: TokenType,
    var: Option<Var>,
    array: Option<Array>,
}

impl VarDecl {
    pub fn new(tc: &mut TokenCollection) -> Self {
        let mut var = Option::None;
        let mut array = Option::None;

        match tc.peek_next_token_type() {
            Some(TokenType::Var) => {
                var = Some(Var::new(tc));
            },
            Some(TokenType::Array) => {
                array = Some(Array::new(tc));
            },
            None => {
                // Compiler Error :
                panic!("Expected variable declaration Var or Array, found EOF.");
            },
            err => {
                // Compiler Error :
                panic!("Expected Var or Array Token, found unexpected Token: {:?}", err);
            },
        }

        VarDecl { node_type: TokenType::VarDecl, var, array }
    }

    pub fn get_value(&self) -> (Option<Var>, Option<Array>)  {
        return (self.var.clone(), self.array.clone())
    }

    pub fn get_type(&self) -> TokenType {
        self.node_type.clone()
    }
}