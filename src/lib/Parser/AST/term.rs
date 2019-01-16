use lib::Lexer::token::TokenCollection;
use lib::Lexer::token::TokenType;
use lib::Lexer::token::Token;
use Parser::AST::factor::Factor;

#[derive(Debug,Clone)]
enum TermList {
    factor(Factor),
    operation(Token),
}

#[derive(Debug,Clone)]
pub struct Term {
    node_type: TokenType,
    term_list: Vec<TermList>,
}

impl Term {
    pub fn new(tc: &mut TokenCollection) -> Self {
        let mut term_list = Vec::new();

        let mut factors : Vec<Factor> = vec!();
        let mut operations : Vec<Token> = vec!();

        term_list.push(TermList::factor(Factor::new(tc)));

        loop {
            //handle MulOp possibility
            match tc.peek_next_token_type() {
                Some(TokenType::MulOp) | Some(TokenType::DivOp) => {
                    //MulOp found, consume then call factor again
                    term_list.push(TermList::operation(tc.get_next_token().unwrap()));
                    term_list.push(TermList::factor(Factor::new(tc)));
                },
                None => {
                    // Compiler Error :
                    panic!("Unexpected EOF in term.");
                },
                _ => {
                    //If no MulOp, return
                    return Term { node_type: TokenType::Term, term_list }
                },
            }
        }
    }

    pub fn get_value(&self) -> Vec<TermList>  {
        return self.term_list.clone()
    }

    pub fn get_type(&self) -> TokenType {
        self.node_type.clone()
    }
}