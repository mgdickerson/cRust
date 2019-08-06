use lib::Utility::syntax_position::Span;
use lib::Lexer::token::{TokenType,Token};
use self::Expr::*;

pub enum Expr {
    Array {
        array_depth: Vec<i64>,
        idents: Vec<Expr>,
    },
    Assign {
        design: Box<Expr>,
        expr: Box<Expr>,
    },
    Comp {
        globals: Vec<Expr>,
        funcs: Vec<Expr>,   // This will include all FuncDecls, and the main function 
                            //denoted by the first {} set without preceeding function ident
    },
    Call {
        func_name: Box<Expr>,
        param: Vec<Expr>,
    },
    Expr {
        l_expr: Box<Expr>,
        r_expr: Box<Expr>,
        math_op: Token,
    },
    FuncBody {
        stmts: Vec<Expr>,
    },
    FuncDecl {
        func_name: Box<Expr>,
        params: Vec<Expr>,
        var_decl: Vec<Expr>,
        func_body: Box<Expr>,
    },
    Ident {
        val: Token,
    },
    If {
        relation: Box<Expr>,
        if_body: Box<Expr>,
        else_body: Option<Box<Expr>>,
    },
    Int {
        val: Token
    },
    Relation {
        l_expr: Box<Expr>,
        r_expr: Box<Expr>,
        rel_op: Box<Token>,
    },
    Var {
        idents: Vec<Expr>,
    },
    While {
        relation: Box<Expr>,
        func_body: Box<Expr>,
    },
}