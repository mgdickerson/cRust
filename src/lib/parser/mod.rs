use std::iter::Peekable;
use std::str::Chars;
use std::vec::IntoIter;

// pub mod AST;
pub mod ast;

use lib::Lexer::token::{Token,TokenType};
use lib::Lexer::Lexer;
use lib::Utility::syntax_position::{BytePos,Span};
use lib::Utility::error::Error;
use lib::parser::ast::{Expr};

// use self::AST::computation;

pub struct Parser {
    token_iter: Peekable<IntoIter<Token>>,
    lo: BytePos,
    hi: BytePos,
    // Not sure what else to put yet.
}

impl<'pctx, 'lxr, 'lctx> Parser {
    fn new(
        iter: &'lxr mut Peekable<Chars<'lctx>>
    ) -> Result<Parser, Error> {
        let token_collection = Lexer::tokenize(iter)?;
        Ok(Parser {
            token_iter: token_collection.into_iter().peekable(),
            lo: BytePos::default(),
            hi: BytePos::default(),
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

    fn advance(&mut self) -> Result<Token, Error> {
        match self.token_iter.next() {
            Some(token) => {
                let tok_span = token.get_span();
                self.hi = tok_span.base() + tok_span.len();
                Ok(token)
            },
            None => Err(Error::Advance)
        }
    }

    pub fn build_ast(
        &mut self,
    ) -> Result<Expr, Error> {
        // Beginning of Analysis, should be main computation. This should consume all comments, 
        // and report error is main declaration is not first.
        while let Ok(token) = self.advance() {
            // Skip comments.
            if let TokenType::Comment(_) = token.peek_type() {
                continue
            }

            // First token encountered (other than comments) should be main.
            if TokenType::Computation == token.peek_type() {
                return self.build_comp()
            } else {
                return Err(Error::MainNF(token))
            }
        }
        
        Err(Error::NoCodeFound)
    }

    pub fn build_comp(
        &mut self,
    ) -> Result<Expr,Error> {
        // Found main(), continue with execution
        

        // TODO : Place Holder
        Err(Error::Eof(self.advance()?))
    }
}
