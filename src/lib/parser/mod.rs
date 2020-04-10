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
    errors: Vec<Error>,
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
            errors: Vec::new(),
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

    fn set_lo(
        &mut self,
        tok: &Token,
    ) {
        self.lo = tok.get_span().base();
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

    fn peek(&mut self) -> Result<Token, Error> {
        match self.token_iter.peek() {
            Some(token) => Ok(token.clone()),
            None => Err(Error::Advance),
        }
    }

    /// When an error is encountered, consume until specified Token is found.
    /// This will allow the parser to move over errors and continue parsing, 
    /// finding more bugs in the process instead of just halting on the first one.
    fn consume_until<F: Fn(Token) -> bool>(
        &mut self,
        pred: F
    ) -> Result<Token, Error> {
        let mut tok = self.advance()?;
        while !pred(tok.clone()) {
            tok = self.advance()?;
        }
        Ok(tok)
    }

    /// Pass lo BytePos to build_span so that saved lo positions can be used.
    fn build_ast_span(
        &mut self,
        lo: BytePos,
    ) -> Result<Span, Error> {
        let span = Span::new(lo, self.hi);
        if let Ok(token) = self.peek() {
            self.lo = token.get_span().base();
            self.hi = self.lo;
        } else {
            self.lo = BytePos::from_usize(0);
            self.hi = self.lo;
        }
        Ok(span)
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
                self.set_lo(&token);
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
        let mut globals : Vec<Expr> = Vec::new();
        let mut funcs : Vec<Expr> = Vec::new();
        let comp_lo = self.lo;

        // Found main(), search for globals
        while let Ok(token) = self.peek() {
            if let TokenType::Var = token.peek_type() {
                let consume_tok = self.advance()?;
            } else if let TokenType::Array = token.peek_type() {
                let consume_tok = self.advance()?;
            } else { break; }
        }

        while let Ok(token) = self.peek() {
            if let TokenType::FuncDecl = token.peek_type() {
                let consume_tok = self.advance()?;
            } else { break; }
        }

        if let Ok(token) = self.advance() {
            if let TokenType::LCurly = token.peek_type() {

            } else { /* error */ }

            if let Ok(token) = self.advance() {
                if let TokenType::ComputationEnd = token.peek_type() {
                    self.set_lo(&token);
                    return Ok(Expr::Comp { globals, funcs, span: self.build_ast_span(comp_lo)? })
                } else {
                    // error
                }
            }
        } else {
            // error
        }

        // TODO : Place Holder
        Err(Error::Eof(self.advance()?))
    }

    pub fn build_var(
        &mut self,
    ) -> Result<Expr,Error> {
        let mut vars = Vec::new();
        let var_lo = self.lo;

        while let Ok(token) = self.advance() {
            if let TokenType::Ident(ident) = token.peek_type() {
                vars.push(Expr::Ident{ val: token.clone(), span: self.build_ast_span(token.get_span().base())? });
                let next_token = self.advance()?;
                match next_token.peek_type() {
                    TokenType::Comma => {
                        // There is likely another ident incoming, continue
                        continue
                    },
                    TokenType::SemiTermination => {
                        // End of this ident, build and return.
                        return Ok(Expr::Var{ idents: vars, span: self.build_ast_span(var_lo)? })
                    },
                    err => {
                        // TODO : add use of consume while and put the error in to self instead of return
                        return Err(Error::UxToken(String::from("',', ';'"), next_token))
                    },
                }
            } else {
                // TODO : add use of consume while and put the error in to self instead of return
                return Err(Error::UxToken(String::from("ident"), token))
            }
        }

        Err(Error::Advance)
    }
}
