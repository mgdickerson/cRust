use std;
use std::iter::Peekable;
use std::str::Chars;
use Lexer;
use lib::Utility::syntax_position::Span;

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    token_type: TokenType,
    span: Span,
}

impl Token {
    /// Standard Token builder.
    pub fn new(token_type: TokenType, span: Span) -> Self {
        Token {
            token_type,
            span,
        }
    }

    /// Returns token type without consuming Token.
    pub fn peek_type(&self) -> TokenType {
        self.token_type.clone()
    }

    /// Consumes Token, returns span
    pub fn get_span(&self) -> Span {
        self.span
    }

    pub fn get_error_message(&self) -> String {
        match self.token_type {
            TokenType::Error(ref string) => string.to_string(),
            _ => String::from("Error in non-error type.")
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    // Null Type, used before type information gathered.
    None,
    Error(String),

    // Basic building block tokens
    Comma,
    SemiTermination,

    // Math Operations
    AddOp,
    SubOp,
    MulOp,
    DivOp,

    // Relative Operations
    EqOp,
    NeqOp,
    LessOp,
    GreaterOp,
    LeqOp,
    GeqOp,

    // Variable Types
    Var,
    Array,

    // Braces
    LCurly,
    RCurly,
    LParen,
    RParen,
    LSquare,
    RSquare,

    // Combination tokens
    Ident(String),
    Number(i64),

    // Statement Kind (Indicated the Let key word)
    Assignment,

    // Conditional Terminators
    If,
    Then,
    Else,
    Fi,
    While,
    Do,
    Od,

    // Function Call Terminators
    FuncDecl,
    FuncCall,
    Return,

    Computation,
    ComputationEnd,

    // Pre-Defined Functions
    InputNum,
    OutputNum,
    OutputNewLine,

    Comment(String),
}

// TODO : Items removed from Token that should actually
// either be IR or dont really have a purpose.
/* 
 * Designator,
 * Factor,
 * Term,
 * Expression,
 * Relation,
 * AssignmentOp,
 * FuncParam,
 * FuncIdent,
 * Statement,
 * StatSequence,
 * TypeDecl,
 * VarDecl,
 * FormalParam,
 * FuncBody,
*/
