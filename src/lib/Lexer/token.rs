use std;
use std::iter::Peekable;
use std::str::Chars;
use std::fmt::Write;
use Lexer;
use lib::Utility::error::Error;
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

    pub fn is_type(&self, tk_ty: &TokenType) -> bool {
        self.token_type == *tk_ty
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

    pub fn invalid_value(&self) -> String {
        let mut err_mssg = String::new();
        write!(err_mssg, "Invalid value was requested for given token: {:?}", self);
        err_mssg
    }

    pub fn get_str_value(&self) -> Result<String, Error> {
        match self.peek_type() {
            TokenType::Error(s) | 
            TokenType::Ident(s) | 
            TokenType::Comment(s) => return Ok(s.clone()),
            _ => return Err(Error::InvalidValueRequest(self.clone())),
        }
    }

    pub fn get_i64_value(&self) -> Result<i64, Error> {
        match self.peek_type() {
            TokenType::Number(n) => return Ok(n),
            _ => return Err(Error::InvalidValueRequest(self.clone())),
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
    LBrace,
    RBrace,
    LParen,
    RParen,
    LBracket,
    RBracket,

    // Combination tokens
    Ident(String),
    Number(i64),

    // Statement Kind (Indicated the Let key word)
    Assignment,
    Arrow,

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