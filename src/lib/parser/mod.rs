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
                if let Some(locals) = self.function_symbols.get(func_name) {
                    if locals.contains(&ident) {
                        self.consume_line()?;
                        return Err(Error::Redef(symbol.clone()))
                    }
                } else {
                    self.function_symbols.insert(func_name.clone(), Vec::new());
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
                if let Some(locals) = self.function_symbols.get(func_name) {
                    if locals.contains(&ident) {
                        return Ok(())
                    }
                } else {
                    match symbol.peek_type() {
                        TokenType::OutputNewLine | TokenType::OutputNum | 
                        TokenType::InputNum => return Ok(()),
                        _ => {},
                    }
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
        self.token_iter = self.token_iter.to_owned().filter_map(|tk| {
            if let TokenType::Comment(_) = tk.peek_type() {
                None
            } else {
                Some(tk.clone())
            }
        } ).collect::<Vec<_>>().into_iter().peekable();

        loop {
            match self.peek()?.peek_type() {
                TokenType::Comment(_) => { self.advance()?; },
                TokenType::Computation => return self.build_comp(),
                _ => return Err(Error::UxToken(String::from("Computation"),self.advance()?))
            }
        }
    }

    pub fn build_comp(
        &mut self,
    ) -> Result<Expr, Error> {
        // Search for global variable declarations
        let comp_lo = self.lo;
        self.ty_checked_advance(TokenType::Computation)?;
        let globals = self.build_var_decl(&None)?;
        let funcs : Vec<Expr> = self.build_func_decls()?;

        self.ty_checked_advance(TokenType::LBrace)?;
        let main_body = self.build_func_body(&None)?;
        self.ty_checked_advance(TokenType::RBrace)?;
        self.ty_checked_advance(TokenType::ComputationEnd)?;

        Ok(Expr::Comp{ globals, funcs, main: Box::new(main_body), span: self.build_ast_span(comp_lo)?})
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
                    let func_lo = self.lo;
                    self.advance()?;
                    let func_ident = self.advance()?;
                    let func_string = Some(self.load_symbols(&func_ident, &None)?);
                    let func_params = self.build_func_param(&func_string)?;
                    let locals = self.build_var_decl(&func_string)?;
                    if let Err(err) = self.ty_checked_advance(TokenType::LBrace) {
                        self.errors.push(err);
                        self.consume_until(|tk| tk.peek_type() == TokenType::RBrace)?;
                    }
                    let body = self.build_func_body(&func_string)?;
                    self.ty_checked_advance(TokenType::RBrace)?;
                    self.ty_checked_advance(TokenType::SemiTermination)?;
                    funcs.push(Expr::FuncDecl{ func_name: Box::new(Expr::Ident{ val: func_ident.clone(), span: func_ident.get_span() }), params: func_params, var_decl: locals, func_body: Box::new(body), span: self.build_ast_span(func_lo)? })
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
        if self.peek()?.peek_type() == TokenType::SemiTermination {
            self.advance()?;
            return Ok(Vec::new())
        }

        self.ty_checked_advance(TokenType::LParen)?;

        if self.peek()?.peek_type() == TokenType::RParen {
            self.advance()?;
            self.ty_checked_advance(TokenType::SemiTermination)?;
            return Ok(Vec::new())
        }

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
                    self.optional_end_take()?;
                    // self.ty_checked_advance(TokenType::SemiTermination)?;

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
        func: &Option<String>,
    ) -> Result<Expr, Error> {
        let body_lo = self.lo;

        let mut stmts: Vec<Expr> = Vec::new();
        loop {
            match self.peek()?.peek_type() {
                TokenType::Assignment => stmts.push(self.build_assignment(func)?),
                TokenType::If => stmts.push(self.build_if(func)?),
                TokenType::While => stmts.push(self.build_while(func)?),
                TokenType::FuncCall => { 
                    stmts.push(self.build_func_call(func)?);
                    self.optional_end_take()?;
                },
                TokenType::Return => stmts.push(self.build_return(func)?),
                _ => break,
            }
        }

        Ok(Expr::FuncBody{ stmts, span: self.build_ast_span(body_lo)? })
    }

    fn build_assignment(
        &mut self,
        func: &Option<String>,
    ) -> Result<Expr, Error> {
        let assign_lo = self.lo;
        self.ty_checked_advance(TokenType::Assignment)?;
        let designator = self.build_designator(func)?;
        self.ty_checked_advance(TokenType::Arrow)?;
        let expr = self.build_expr(func)?;
        self.optional_end_take()?;
        Ok(Expr::Assign{ design: Box::new(designator), expr: Box::new(expr), span: self.build_ast_span(assign_lo)? })
    }

    fn optional_end(
        &mut self,
    ) -> Result<(), Error> {
        match self.peek()?.peek_type() {
            TokenType::Fi | TokenType::Od |
            TokenType::RBrace | TokenType::Else |
            TokenType::RBracket | TokenType::AddOp |
            TokenType::SubOp | TokenType::MulOp |
            TokenType::DivOp | TokenType::RParen |
            TokenType::SemiTermination => Ok(()),
            _ => {Err(Error::UxToken(String::from("fi, od, '}', else, ';'"), self.peek()?))},
        }
    }

    fn optional_end_take(
        &mut self
    ) -> Result<(), Error> {
        match self.peek()?.peek_type() {
            TokenType::Fi | TokenType::Od |
            TokenType::RBrace | TokenType::Else |
            TokenType::RBracket | TokenType::AddOp |
            TokenType::SubOp | TokenType::MulOp |
            TokenType::DivOp | TokenType::RParen => Ok(()),
            TokenType::SemiTermination => {
                self.advance()?;
                Ok(())
            },
            _ => Err(Error::UxToken(String::from("fi, od, '}', else, ';'"), self.advance()?)),
        }
    }

    fn build_if(
        &mut self,
        func: &Option<String>,
    ) -> Result<Expr, Error> {
        let if_lo = self.lo;
        self.ty_checked_advance(TokenType::If)?;
        let rel_op = self.build_relation(func)?;
        self.ty_checked_advance(TokenType::Then)?;
        
        let if_body = self.build_func_body(func)?;

        if self.peek()?.peek_type() == TokenType::Else {
            self.advance()?;
            let else_body = self.build_func_body(func)?;
            self.ty_checked_advance(TokenType::Fi)?;
            self.optional_end_take()?;
            Ok(Expr::If{ relation: Box::new(rel_op), if_body: Box::new(if_body), else_body: Some(Box::new(else_body)), span: self.build_ast_span(if_lo)? })
        } else {
            self.ty_checked_advance(TokenType::Fi)?;
            self.optional_end_take()?;
            Ok(Expr::If{ relation: Box::new(rel_op), if_body: Box::new(if_body), else_body: None, span: self.build_ast_span(if_lo)? })
        }
    }

    fn build_while(
        &mut self,
        func: &Option<String>
    ) -> Result<Expr, Error> {
        let while_lo = self.lo;
        self.ty_checked_advance(TokenType::While)?;
        let rel_op = self.build_relation(func)?;
        self.ty_checked_advance(TokenType::Do)?;

        let body = self.build_func_body(func)?;

        self.ty_checked_advance(TokenType::Od)?;
        self.optional_end_take()?;
        Ok(Expr::While{ relation: Box::new(rel_op), func_body: Box::new(body), span: self.build_ast_span(while_lo)? })
    }

    fn build_func_call(
        &mut self,
        func: &Option<String>,
    ) -> Result<Expr, Error> {
        let func_lo = self.lo;
        self.ty_checked_advance(TokenType::FuncCall)?;
        let ident = self.advance()?;
        match ident.peek_type() {
            TokenType::OutputNum | TokenType::OutputNewLine |
            TokenType::InputNum => { /* Pre-defined function, do nothing */ },
            _ => self.check_symbol(&ident, func)?,
        }

        if self.optional_end() == Ok(()) {
            return Ok(Expr::Call{ func_name: Box::new(Expr::Ident{ val: ident.clone(), span: ident.get_span()}), args: Vec::new(), span: self.build_ast_span(func_lo)? })
        }

        self.ty_checked_advance(TokenType::LParen)?;

        if self.peek()?.peek_type() == TokenType::RParen {
            self.advance()?;
            self.optional_end()?;
            return Ok(Expr::Call{ func_name: Box::new(Expr::Ident{ val: ident.clone(), span: ident.get_span()}), args: Vec::new(), span: self.build_ast_span(func_lo)? })
        }

        let mut expr_list = Vec::new();
        loop {
            expr_list.push(self.build_expr(func)?);
            match self.peek()?.peek_type() {
                TokenType::Comma => {
                    self.advance()?;
                    continue;
                },
                TokenType::RParen => {
                    self.advance()?;
                    break;
                },
                _ => return Err(Error::UxToken(String::from("',', ')'"), self.advance()?)),
            }
        }

        self.optional_end()?;
        Ok(Expr::Call{ func_name: Box::new(Expr::Ident{ val: ident.clone(), span: ident.get_span()}), args: expr_list, span: self.build_ast_span(func_lo)? })
    }

    fn build_return(
        &mut self,
        func: &Option<String>,
    ) -> Result<Expr, Error> {
        let ret_lo = self.lo;
        self.ty_checked_advance(TokenType::Return)?;
        let ret_expr = self.build_expr(func)?;
        self.optional_end()?;
        Ok(Expr::Return{ expr: Box::new(ret_expr), span: self.build_ast_span(ret_lo)? })
    }

    fn build_designator(
        &mut self,
        func: &Option<String>,
    ) -> Result<Expr, Error> {
        let des_lo = self.lo;
        let symbol = self.advance()?;
        self.check_symbol(&symbol, func)?;

        if self.peek()?.peek_type() == TokenType::LBracket {
            Ok(Expr::Design{ ident: Box::new(Expr::Ident{ val: symbol.clone(), span: symbol.get_span()}), exprs: self.build_array_expr_list(func)?, span: self.build_ast_span(des_lo)? })
        } else {
            Ok(Expr::Design{ ident: Box::new(Expr::Ident{ val: symbol.clone(), span: symbol.get_span()}), exprs: Vec::new(), span: self.build_ast_span(des_lo)? })
        }
    }

    fn build_array_expr_list(
        &mut self,
        func: &Option<String>,
    ) -> Result<Vec<Expr>, Error> {
        let mut expr_list = Vec::new();
        loop {
            self.ty_checked_advance(TokenType::LBracket)?;
            expr_list.push(self.build_expr(func)?);
            self.ty_checked_advance(TokenType::RBracket)?;
            
            if self.peek()?.peek_type() != TokenType::LBracket {
                break;
            }
        }

        Ok(expr_list)
    }

    fn build_expr(
        &mut self,
        func: &Option<String>,
    ) -> Result<Expr, Error> {
        let expr_lo = self.lo;
        let lexpr = self.build_term(func)?;

        match self.peek()?.peek_type() {
            TokenType::AddOp | TokenType::SubOp => {
                let math_op = Some(self.advance()?);
                Ok(Expr::Expr{ l_expr: Box::new(lexpr), r_expr: Some(Box::new(self.build_expr(func)?)), math_op, span: self.build_ast_span(expr_lo)? })
            },
            _ => Ok(Expr::Expr{ l_expr: Box::new(lexpr), r_expr: None, math_op: None, span: self.build_ast_span(expr_lo)? }),
        }
    }

    fn build_term(
        &mut self,
        func: &Option<String>,
    ) -> Result<Expr, Error> {
        let term_lo = self.lo;
        let lfact = self.build_factor(func)?;

        match self.peek()?.peek_type() {
            TokenType::MulOp | TokenType::DivOp => {
                let math_op = Some(self.advance()?);
                Ok(Expr::Expr{ l_expr: Box::new(lfact), r_expr: Some(Box::new(self.build_term(func)?)), math_op, span: self.build_ast_span(term_lo)? })
            },
            _ => {
                Ok(Expr::Expr{ l_expr: Box::new(lfact), r_expr: None, math_op: None, span: self.build_ast_span(term_lo)? })
            }
        }
    }

    fn build_factor(
        &mut self,
        func: &Option<String>,
    ) -> Result<Expr, Error> {
        let factor = self.peek()?;
        match factor.peek_type() {
            TokenType::Ident(_) => self.build_designator(func),
            TokenType::Number(_) => Ok(Expr::Int{ val: self.advance()?, span: factor.get_span() }),
            TokenType::FuncCall => self.build_func_call(func),
            TokenType::LParen => {
                self.advance()?;
                let expr = self.build_expr(func)?;
                self.ty_checked_advance(TokenType::RParen)?;
                Ok(expr)
            },
            _ => {
                self.consume_line()?;
                Err(Error::UxToken(String::from("designator, number, function call, or '('"), factor))
            },
        }
    }

    fn build_relation(
        &mut self,
        func: &Option<String>,
    ) -> Result<Expr, Error> {
        let rel_lo = self.lo;
        let lexpr = self.build_expr(func)?;
        let rel_op = self.advance()?;
        match rel_op.peek_type() {
            TokenType::EqOp |
            TokenType::NeqOp |
            TokenType::LessOp |
            TokenType::GreaterOp |
            TokenType::LeqOp |
            TokenType::GeqOp => {},
            _ => return Err(Error::UxToken(String::from("==, !=, <, <=, >, >="), rel_op)),
        }
        Ok(Expr::Relation{ l_expr: Box::new(lexpr), r_expr: Box::new(self.build_expr(func)?), rel_op: rel_op, span: self.build_ast_span(rel_lo)? })
    }
}
