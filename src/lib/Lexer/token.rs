use std;
use Lexer;

#[derive(Debug, Clone)]
pub struct Token {
    token_type: TokenType,
    token_contents: String,
}

impl Token {
    pub fn new(TkTy: TokenType, TkCn: String) -> Self {
        Token {
            token_type: TkTy,
            token_contents: TkCn,
        }
    }

    pub fn get_contents(&self) -> String {
        let copy = self.token_contents.clone();
        copy
    }

    pub fn get_type(&self) -> TokenType {
        let copy = self.token_type.clone();
        copy
    }

    pub fn peek_type(&self) -> TokenType {
        let peek_token_copy = self.token_type.clone();
        peek_token_copy
    }

    pub fn peek_content(&self) -> String {
        let peek_content_copy = self.token_contents.clone();
        peek_content_copy
    }
}

#[derive(Debug)]
pub struct TokenCollection {
    token_vector: std::iter::Peekable<std::vec::IntoIter<Token>>,
}

impl TokenCollection {
    pub fn collect(iter: &mut std::iter::Peekable<std::str::Chars<'_>>) -> TokenCollection {
        let mut token_builder: Vec<Token> = Vec::new();

        loop {
            if iter.peek() == None {
                return TokenCollection {
                    token_vector: token_builder.into_iter().peekable(),
                };
            }

            if let Some(token) = Lexer::get_token(iter) {
                token_builder.push(token);
            } else {
                //Lexer::get_token(iter) returned None.
            }
        }
    }

    //  Debugging function
    pub fn get_vector(self) -> std::iter::Peekable<std::vec::IntoIter<Token>> {
        self.token_vector
    }

    pub fn get_next_token(&mut self) -> Option<Token> {
        self.token_vector.next()
    }

    pub fn peek_next_token_type(&mut self) -> Option<TokenType> {
        match self.token_vector.peek() {
            Some(x) => {
                let token_type_peek = x.clone();
                Some(token_type_peek.peek_type())
            }
            None => None,
        }
    }
}
//std::iter::Peekable<std::slice::Iter<Token<'_>>>
#[derive(Debug, Copy, Clone)]
pub enum TokenType {
    // Debugging Type
    Test,

    // Null Type, used before type information gathered.
    None,

    // Basic building block tokens
    Letter,
    Digit,
    Comma,
    SemiTermination,

    // Operations
    RelOp,
    MathOp,
    AddOp,
    SubOp,
    MulOp,
    DivOp,

    // Variable Types
    Var,
    Array,

    // Braces
    LeftBrace,
    RightBrace,
    LeftPara,
    RightPara,
    LeftBracket,
    RightBracket,
    // TODO : Need to differentiate

    // Combination tokens
    Ident,
    Number,

    Designator,
    Factor,
    Term,
    Expression,
    Relation,

    Assignment,
    AssignmentOp,

    FuncCall,
    FuncParam,
    FuncIdent,

    IfStatement,
    ThenStatement,
    ElseStatement,
    FiStatement,

    WhileStatement,
    DoStatement,
    OdStatement,

    ReturnStatement,

    Statement,
    StatSequence,

    TypeDecl,
    VarDecl,
    FuncDecl,
    FormalParam,
    FuncBody,
    Computation,
    ComputationEnd,

    InputNum,
    OutputNum,
    OutputNewLine,

    Comment,
}
