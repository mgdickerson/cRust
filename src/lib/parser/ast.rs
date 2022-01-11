use self::Expr::*;
use lib::Lexer::token::{Token, TokenType};
use lib::Utility::syntax_position::Span;
use lib::Utility::Dummy;
use std::fmt::{self, Display, Formatter};

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
        funcs: Vec<Expr>, // This will include all FuncDecls, and the main function
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
    },
}

impl Default for Expr {
    fn default() -> Self {
        Expr::Comp {
            globals: Vec::default(),
            funcs: Vec::default(),
            main: Box::new(Expr::dummy()),
            span: Span::default(),
        }
    }
}

impl Dummy for Expr {
    fn dummy() -> Self {
        Expr::Error {
            consumed_tokens: Vec::default(),
            span: Span::default(),
        }
    }
}

impl Display for Expr {
    fn fmt(&self, formatter: &mut Formatter) -> fmt::Result {
        self.printer(formatter, 1)
    }
}

impl Expr {
    /// Pretty Printing function for inspecting the AST
    fn printer(&self, fmt: &mut Formatter, indent: usize) -> fmt::Result {
        let offset_str = std::iter::repeat("  ").take(indent).collect::<String>();
        let nxt_offset = std::iter::repeat("  ").take(indent + 1).collect::<String>();
        match self {
            Array {
                array_depth,
                idents,
                span,
            } => {
                write!(fmt, "{}Array {{\n{}offsets: [", offset_str, nxt_offset);
                for (count, tok) in array_depth.iter().enumerate() {
                    if count == 0 {
                        write!(fmt, "{}", tok.get_i64_value().unwrap());
                    } else {
                        write!(fmt, ", {}", tok.get_i64_value().unwrap());
                    }
                }
                write!(fmt, "],\n{}idents: [", nxt_offset);
                for (count, ident) in idents.iter().enumerate() {
                    ident.printer(fmt, indent + 2);
                }
                write!(
                    fmt,
                    "{}],\n{}span: {:?},\n{}}}\n",
                    nxt_offset, nxt_offset, span, offset_str
                )
            }
            Assign { design, expr, span } => {
                write!(fmt, "{}Assign {{\n{}lhs:\n", offset_str, nxt_offset);
                design.printer(fmt, indent + 2);
                write!(fmt, "{}rhs:\n", nxt_offset);
                expr.printer(fmt, indent + 2);
                write!(fmt, "{}span: {:?},\n{}}}\n", nxt_offset, span, offset_str)
            }
            Comp {
                globals,
                funcs,
                main,
                span,
            } => {
                write!(fmt, "{}Comp {{\n{}globals:\n", offset_str, nxt_offset);
                for global in globals {
                    global.printer(fmt, indent + 2);
                }
                write!(fmt, "{}functions:\n", nxt_offset);
                for func in funcs {
                    func.printer(fmt, indent + 2);
                }
                write!(fmt, "{}main:\n", nxt_offset);
                main.printer(fmt, indent + 2);
                write!(fmt, "{}span: {:?}\n{}}}\n", nxt_offset, span, offset_str)
            }
            Call {
                func_name,
                args,
                span,
            } => {
                write!(fmt, "{}FuncCall {{\n{}func_name:\n", offset_str, nxt_offset);
                func_name.printer(fmt, indent + 2);
                write!(fmt, "{}args:\n", nxt_offset);
                for arg in args {
                    arg.printer(fmt, indent + 2);
                }
                write!(fmt, "{}span: {:?},\n{}}}\n", nxt_offset, span, offset_str)
            }
            Design { ident, exprs, span } => {
                write!(fmt, "{}Designator: {{\n{}ident:\n", offset_str, nxt_offset);
                ident.printer(fmt, indent + 2);
                write!(fmt, "{}exprs:\n", nxt_offset);
                for expr in exprs {
                    expr.printer(fmt, indent + 2);
                }
                write!(fmt, "{}span: {:?}\n{}}}\n", nxt_offset, span, offset_str)
            }
            Expr::Expr {
                l_expr,
                r_expr,
                math_op,
                span,
            } => {
                write!(fmt, "{}Expr {{\n{}l_expr:\n", offset_str, nxt_offset);
                l_expr.printer(fmt, indent + 2);
                if let Some(r_expr) = r_expr {
                    write!(
                        fmt,
                        "{}op: {}\n",
                        nxt_offset,
                        math_op.clone().unwrap().get_str_value().unwrap()
                    );
                    write!(fmt, "{}r_expr:\n", nxt_offset);
                    r_expr.printer(fmt, indent + 2);
                }
                write!(fmt, "{}span: {:?},\n{}}}\n", nxt_offset, span, offset_str)
            }
            FuncBody { stmts, span } => {
                write!(fmt, "{}FuncBody {{\n{}stmts:\n", offset_str, nxt_offset);
                for stmt in stmts {
                    stmt.printer(fmt, indent + 2);
                }
                write!(fmt, "{}span: {:?}\n{}}}\n", nxt_offset, span, offset_str)
            }
            FuncDecl {
                func_name,
                params,
                var_decl,
                func_body,
                span,
            } => {
                write!(fmt, "{}FuncDecl {{\n{}func_name:\n", offset_str, nxt_offset);
                func_name.printer(fmt, indent + 2);
                write!(fmt, "{}params:\n", nxt_offset);
                for param in params {
                    param.printer(fmt, indent + 2);
                }
                write!(fmt, "{}var_declarations:\n", nxt_offset);
                for var in var_decl {
                    var.printer(fmt, indent + 2);
                }
                write!(fmt, "{}func_body:\n", nxt_offset);
                func_body.printer(fmt, indent + 2);
                write!(fmt, "{}span: {:?},\n{}}}\n", nxt_offset, span, offset_str)
            }
            Ident { val, span } => write!(fmt, "{}{}\n", offset_str, val.get_str_value().unwrap()),
            If {
                relation,
                if_body,
                else_body,
                span,
            } => {
                write!(fmt, "{}If {{\n{}relation:\n", offset_str, nxt_offset);
                relation.printer(fmt, indent + 2);
                write!(fmt, "{}body:\n", nxt_offset);
                if_body.printer(fmt, indent + 2);
                if let Some(el) = else_body {
                    write!(fmt, "{}else:\n", nxt_offset);
                    el.printer(fmt, indent + 2);
                }
                write!(fmt, "{}span: {:?},{}}}\n", nxt_offset, span, offset_str)
            }
            Int { val, span } => write!(fmt, "{}{}\n", offset_str, val.get_i64_value().unwrap()),
            Relation {
                l_expr,
                r_expr,
                rel_op,
                span,
            } => {
                write!(fmt, "{}Relation {{\n{}l_expr:\n", offset_str, nxt_offset);
                l_expr.printer(fmt, indent + 2);
                write!(
                    fmt,
                    "{}rel_op: {}\n",
                    nxt_offset,
                    rel_op.get_str_value().unwrap()
                );
                write!(fmt, "{}r_expr:\n", nxt_offset);
                r_expr.printer(fmt, indent + 2);
                write!(fmt, "{}span: {:?},\n{}}}\n", nxt_offset, span, offset_str)
            }
            Return { expr, span } => {
                write!(fmt, "{}Return {{\n{}expr:\n", offset_str, nxt_offset);
                expr.printer(fmt, indent + 2);
                write!(fmt, "{}span: {:?},\n{}}}\n", offset_str, span, nxt_offset)
            }
            Var { idents, span } => {
                write!(fmt, "{}Variables {{\n{}idents:\n", offset_str, nxt_offset);
                for ident in idents {
                    ident.printer(fmt, indent + 2);
                }
                write!(fmt, "{}span: {:?},\n{}}}\n", nxt_offset, span, offset_str)
            }
            While {
                relation,
                func_body,
                span,
            } => {
                write!(fmt, "{}While {{\n{}relation:\n", offset_str, nxt_offset);
                relation.printer(fmt, indent + 2);
                write!(fmt, "{}body:\n", nxt_offset);
                func_body.printer(fmt, indent + 2);
                write!(fmt, "{}span: {:?},\n{}}}\n", nxt_offset, span, offset_str)
            }
            Error {
                consumed_tokens,
                span,
            } => write!(
                fmt,
                "{}Error {{\n{}error: {:?}\n{}}}\n",
                offset_str, nxt_offset, self, offset_str
            ),
        }
    }
}
