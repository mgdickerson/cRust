use std::iter::Peekable;
use std::str::Chars;
use std::vec::IntoIter;

// pub mod AST;
pub mod ast;

use lib::Lexer::token::Token;
use lib::Lexer::Lexer;
use lib::Utility::error::Error;
// use self::AST::computation;

pub struct Parser {
    token_iter: Peekable<IntoIter<Token>>,
    // Not sure what else to put yet.
}

impl<'pctx, 'lxr, 'lctx> Parser {
    fn new(
        iter: &'lxr mut Peekable<Chars<'lctx>>
    ) -> Result<Parser, Error> {
        let token_collection = Lexer::tokenize(iter)?;
        Ok(Parser {
            token_iter: token_collection.into_iter().peekable(),
        })
    }

    // TODO : Currently returns (), but should eventually return some sort of AST structure 
    // that has been parsed and can be walked.
    pub fn parse(
        iter: &'lxr mut Peekable<Chars<'lctx>>
    ) -> Result<(), Error> {
        let parser = Parser::new(iter)?;

        // TODO : Proves Parser does something.
        for item in parser.token_iter {
            println!("{:?}", item);
        }

        // TODO : Temp result to keep parser from complaining on compile.
        Ok(())
    }
}
