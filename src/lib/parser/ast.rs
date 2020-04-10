use lib::Utility::syntax_position::Span;
use lib::Lexer::token::{TokenType,Token};
use self::Expr::*;

pub enum Expr {
    Array {
        array_depth: Vec<i64>,
        idents: Vec<Expr>,
        span: Span,
    },
    Assign {
        design: Box<Expr>,
        expr: Box<Expr>,
    },
    Comp {
        globals: Vec<Expr>,
        funcs: Vec<Expr>,   // This will include all FuncDecls, and the main function 
                            //denoted by the first {} set without preceeding function ident
        span: Span,
    },
    Call {
        func_name: Box<Expr>,
        param: Vec<Expr>,
        span: Span,
    },
    Expr {
        l_expr: Box<Expr>,
        r_expr: Box<Expr>,
        math_op: Token,
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
        rel_op: Box<Token>,
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
}

impl Default for Expr {
    fn default() -> Self { Expr::Comp{ globals: Vec::default(), funcs: Vec::default(), span: Span::default() } }
}