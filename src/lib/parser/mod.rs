use std::iter::Peekable;
use std::str::Chars;
use std::vec::IntoIter;
use std::collections::HashMap;
use std::fmt::Write;

// pub mod AST;
pub mod ast;

use lib::Lexer::token::{Token,TokenType};
use lib::Lexer::Lexer;
use lib::Utility::syntax_position::{BytePos,Span};
use lib::Utility::error::Error;
use lib::Utility::Dummy;
use lib::parser::ast::{Expr};

// use self::AST::computation;

pub struct Parser {
    token_iter: Peekable<IntoIter<Token>>,
    errors: Vec<Error>,
    lo: BytePos,
    hi: BytePos,
    global_symbols: Vec<String>,
    function_symbols: HashMap<String, Vec<String>>,
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
            global_symbols: Vec::new(),
            function_symbols: HashMap::new(),
        })
    }

    // TODO : Currently returns (), but should eventually return some sort of AST structure 
    // that has been parsed and can be walked.
    pub fn parse(
        iter: &'lxr mut Peekable<Chars<'lctx>>
    ) -> Result<(), Error> {
        let mut parser = Parser::new(iter)?;

        // TODO : Proves Parser does something.
        // for item in parser.token_iter {
        //     println!("{:?}", item);
        // }
        match parser.build_ast() {
            Ok(_) => println!("Parse OK!"),
            Err(err) => {
                // Likely Multiple Errors, first print out collected errors.
                parser.errors.push(err);
                return Err(Error::ParsingError(parser.errors))
            }
        };

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

    fn ty_checked_advance(
        &mut self,
        tk_ty: TokenType,
    ) -> Result<Token, Error> {
        match self.token_iter.next() {
            Some(token) => {
                let tok_span = token.get_span();
                self.hi = tok_span.base() + tok_span.len();
                if token.is_type(&tk_ty) {
                    Ok(token)
                } else {
                    let mut err_mssg = String::new();
                    write!(err_mssg, "{:?}", tk_ty);
                    Err(Error::UxToken(err_mssg, token))
                }
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
    ) -> Result<Expr, Error> {
        let mut tok = self.advance()?;
        while !pred(tok.clone()) {
            tok = self.advance()?;
        }
        Ok(Expr::dummy())
    }

    fn consume_line(
        &mut self,
    ) -> Result<Expr, Error> {
        self.consume_until(|tk| tk.peek_type() == TokenType::SemiTermination)
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

    // If func option is None, we are checking global scope.
    fn load_symbols(
        &mut self,
        symbol: &Token,
        func: &Option<String>,
    ) -> Result<String, Error> {
        if let TokenType::Ident(ident) = symbol.peek_type() {
            if let Some(func_name) = func {
                if self.function_symbols.get(func_name).unwrap().contains(&ident) {
                    self.consume_line()?;
                    return Err(Error::Redef(symbol.clone()))
                }
            }
            
            if self.global_symbols.contains(&ident) {
                self.consume_line()?;
                return Err(Error::Redef(symbol.clone()))
            }

            // Not contained within the set of global symbols, add to the set
            if let Some(func_name) = func {
                self.function_symbols.get_mut(func_name).unwrap().push(ident.clone());
            } else {
                self.global_symbols.push(ident.clone());
            }
            Ok(ident)
        } else {
            self.consume_line()?;
            return Err(Error::UxToken(String::from("ident"), symbol.clone()))
        }
    }

    fn check_symbol(
        &mut self,
        symbol: &Token,
        func: &Option<String>,
    ) -> Result<(), Error> {
        if let TokenType::Ident(ident) = symbol.peek_type() {
            if let Some(func_name) = func {
                if self.function_symbols.get(func_name).unwrap().contains(&ident) {
                    return Ok(())
                }
            }
            
            if self.global_symbols.contains(&ident) {
                return Ok(())
            }

            // Not contained within either func or global idents
            return Err(Error::UndefIdent(symbol.clone()));
        } else {
            self.consume_line()?;
            return Err(Error::UxToken(String::from("ident"), symbol.clone()))
        }
    }

    pub fn build_ast(
        &mut self,
    ) -> Result<Expr, Error> {
        // Beginning of Analysis, should be main computation. This should consume all comments, 
        // and report error if main declaration is not first.
        while let Ok(token) = self.advance() {
            // Skip comments.
            if let TokenType::Comment(_) = token.peek_type() {
                continue
            }

            // First token encountered (other than comments) should be main.
            if TokenType::Computation == token.peek_type() {
                let main_lo = token.get_span().base();
                let next_tok = self.peek()?;
                self.set_lo(&next_tok);
                return self.build_comp()
            } else {
                return Err(Error::MainNF(token))
            }
        }
        
        Err(Error::NoCodeFound)
    }

    pub fn build_comp(
        &mut self,
    ) -> Result<Expr, Error> {
        // Search for global variable declarations
        let comp_lo = self.lo;
        let globals = self.build_var_decl(&None)?;
        let mut funcs : Vec<Expr> = self.build_func_decls()?;

        println!("Current set found:\n{:?}", globals);

        

        if let Ok(token) = self.advance() {
            if let TokenType::LBrace = token.peek_type() {

            } else { 
                self.errors.push(Error::UxToken(String::from("{"), token));
                // TODO : Handle Error, find out if I am doing numbering incorrectly.
            }

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

    pub fn build_var_decl(
        &mut self,
        func: &Option<String>,
    ) -> Result<Vec<Expr>,Error> {
        let mut vars = Vec::new();
        loop {
            let token = self.peek()?;
            match token.peek_type() {
                TokenType::Array => {
                    self.advance()?;
                    vars.push(self.build_array(func)?);
                },
                TokenType::Var => {
                    self.advance()?;
                    vars.push(self.build_var(func)?);
                },
                _ => {
                    break;
                },
            }
        }

        return Ok(vars)
    }

    pub fn build_var(
        &mut self,
        func: &Option<String>,
    ) -> Result<Expr, Error> {
        let var_lo = self.lo;
        return Ok(Expr::Var{ idents: self.pull_idents_list(func)?, span: self.build_ast_span(var_lo)? });
    }

    pub fn build_array(
        &mut self,
        func: &Option<String>,
    ) -> Result<Expr, Error> {
        let mut arrayDepth = Vec::new();
        let array_lo = self.lo;

        loop {
            let token = self.advance()?;
            match token.peek_type() {
                TokenType::LBracket => {
                    // grab depth digit
                    let depth = self.advance()?;
                    if let TokenType::Number(_num) = depth.peek_type() {
                        arrayDepth.push(depth);
                    } else {
                        self.errors.push(Error::UxToken(String::from("'[#number]'"), token));
                        return self.consume_until(|tk| tk.peek_type() == TokenType::SemiTermination)
                    }

                    // Grab closing bracket
                    if TokenType::RBracket != self.advance()?.peek_type() {
                        self.errors.push(Error::UxToken(String::from("']'"), token));
                        return self.consume_until(|tk| tk.peek_type() == TokenType::SemiTermination)
                    }

                    // Break if next token is ident, otherwise continue loop.
                    if let TokenType::Ident(_) = self.peek()?.peek_type() {
                        return Ok(Expr::Array{ array_depth: arrayDepth, idents: self.pull_idents_list(func)?, span: self.build_ast_span(array_lo)? });
                    }
                }
                _ => {
                    self.errors.push(Error::UxToken(String::from("'[', 'ident'"), token));
                    return self.consume_until(|tk| tk.peek_type() == TokenType::SemiTermination)
                }
            }
        }
    }

    fn pull_idents_list(
        &mut self,
        func: &Option<String>
    ) -> Result<Vec<Expr>, Error> {
        let mut idents = Vec::new();

        loop {
            let token = self.advance()?;
            self.load_symbols(&token, func)?;
            
            idents.push(Expr::Ident{ val: token.clone(), span: self.build_ast_span(token.get_span().base())? });
            
            let next_token = self.peek()?;
            match next_token.peek_type() {
                TokenType::Comma => {
                    // There is likely another ident incoming, continue
                    self.advance()?;
                    continue
                },
                TokenType::SemiTermination => {
                    // End of this ident, build and return.
                    self.advance()?;
                    return Ok(idents);
                },
                err => {
                    self.consume_line()?;
                    return Err(Error::UxToken(String::from("',', ';'"), next_token))
                },
            }
        }
    }

    fn build_func_decls(
        &mut self,
    ) -> Result<Vec<Expr>, Error> {
        let mut funcs = Vec::new();

        loop {
            let token = self.peek()?;
            match token.peek_type() {
                TokenType::FuncDecl => {
                    self.advance()?;
                    let func_ident = self.advance()?;
                    let func_string = Some(self.load_symbols(&func_ident, &None)?);
                    let func_params = self.build_func_param(&func_string);
                    let locals = self.build_var_decl(&func_string)?;
                    // TODO : I believe this change removed requirement of sending Optional Locals.
                    self.build_func_body(Some(&locals), &func_string);
                    // TODO : Func Body
                },
                _ => {
                    break;
                },
            }
        }

        return Ok(funcs);
    }

    fn build_func_param(
        &mut self,
        func: &Option<String>,
    ) -> Result<Vec<Expr>, Error> {
        self.ty_checked_advance(TokenType::LParen)?;

        let mut params = Vec::new();
        loop {
            let token = self.advance()?;
            self.load_symbols(&token, func)?;
            
            params.push(Expr::Ident{ val: token.clone(), span: self.build_ast_span(token.get_span().base())? });
            
            let next_token = self.peek()?;
            match next_token.peek_type() {
                TokenType::Comma => {
                    // There is likely another ident incoming, continue
                    self.advance()?;
                    continue
                },
                TokenType::RParen => {
                    // End of this ident, build and return.
                    self.advance()?;
                    self.ty_checked_advance(TokenType::SemiTermination)?;

                    return Ok(params);
                },
                err => {
                    self.consume_line()?;
                    return Err(Error::UxToken(String::from("',', ')'"), next_token))
                },
            }
        }
    }

    fn build_func_body(
        &mut self,
        locals: Option<&Vec<Expr>>,
        func: &Option<String>,
    ) -> Result<Expr, Error> {
        if let Err(err) = self.ty_checked_advance(TokenType::LBrace) {
            self.errors.push(err);
            self.consume_until(|tk| tk.peek_type() == TokenType::RBrace)?;
        }

        let mut stmts: Vec<Expr> = Vec::new();
        loop {
            match self.peek()?.peek_type() {
                TokenType::Assignment => {},
                TokenType::If => {},
                TokenType::While => {},
                TokenType::FuncCall => {},
                TokenType::Return => {},
                _ => break,
            }
        }

        println!("Value of statemnets: {:?}", stmts);

        return Ok(Expr::default())
    }

    fn build_assignment(
        &mut self,
        locals: Option<&Vec<Expr>>,
        func: &Option<String>,
    ) -> Result<Expr, Error> {
        self.ty_checked_advance(TokenType::Assignment)?;

        // Temp
        Err(Error::NoCodeFound)
    }

    fn build_designator(
        &mut self,
        func: &Option<String>,
    ) -> Result<Expr, Error> {
        let symbol = self.advance()?;
        self.check_symbol(&symbol, func)?;


        loop {
            let token = self.peek()?;

            match token.peek_type() {
                TokenType::LBrace => {},
                _ => break,
            }
        }

        // Temp
        Err(Error::NoCodeFound)
    }

    fn build_array_expr_list(
        &mut self,
        func: &Option<String>,
    ) -> Result<Vec<Expr>, Error> {
        loop {
            self.ty_checked_advance(TokenType::LBracket)?;

            // TODO build expression
        }
    }
}
