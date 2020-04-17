use lib::Utility::syntax_position::Span;
use lib::Utility::Dummy;
use lib::Lexer::token::{TokenType,Token};
use self::Expr::*;

#[derive(Debug)]
pub enum Expr {
    Array {
        array_depth: Vec<Token>,
        idents: Vec<Expr>,
        span: Span,
    },
    Assign {
        design: Box<Expr>,
        expr: Box<Expr>,
        span: Span,
    },
    Comp {
        globals: Vec<Expr>,
        funcs: Vec<Expr>,   // This will include all FuncDecls, and the main function 
                            //denoted by the first {} set without preceeding function ident
        main: Box<Expr>,
        span: Span,
    },
    Call {
        func_name: Box<Expr>,
        args: Vec<Expr>,
        span: Span,
    },
    Design {
        ident: Box<Expr>,
        exprs: Vec<Expr>,
        span: Span,
    },
    Expr {
        l_expr: Box<Expr>,
        r_expr: Option<Box<Expr>>,
        math_op: Option<Token>,
        span: Span,
    },
    FuncBody {
        stmts: Vec<Expr>,
        span: Span,
    },
    FuncDecl {
        func_name: Box<Expr>,
        params: Vec<Expr>,
        var_decl: Vec<Expr>,
        func_body: Box<Expr>,
        span: Span,
    },
    Ident {
        val: Token,
        span: Span,
    },
    If {
        relation: Box<Expr>,
        if_body: Box<Expr>,
        else_body: Option<Box<Expr>>,
        span: Span,
    },
    Int {
        val: Token,
        span: Span,
    },
    Relation {
        l_expr: Box<Expr>,
        r_expr: Box<Expr>,
        rel_op: Token,
        span: Span,
    },
    Return {
        expr: Box<Expr>,
        span: Span,
    },
    Var {
        idents: Vec<Expr>,
        span: Span,
    },
    While {
        relation: Box<Expr>,
        func_body: Box<Expr>,
        span: Span,
    },
    Error {
        consumed_tokens: Vec<Token>,
        span: Span,
    }
}

impl Default for Expr {
    fn default() -> Self { Expr::Comp{ globals: Vec::default(), funcs: Vec::default(), main: Box::new(Expr::dummy()), span: Span::default() } }
}

impl Dummy for Expr {
    fn dummy() -> Self { Expr::Error{ consumed_tokens: Vec::default(), span: Span::default() }}
}