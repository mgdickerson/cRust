use lib::Lexer::token::TokenCollection;
use lib::Lexer::token::TokenType;
use Parser::AST::assignment::Assignment;
use Parser::AST::if_stmt::IfStmt;
use Parser::AST::while_stmt::WhileStmt;
use Parser::AST::func_call::FuncCall;
use Parser::AST::return_stmt::ReturnStmt;

#[derive(Debug,Clone)]
pub struct FuncBody {
    node_type: TokenType,
    assignment: Vec<Assignment>,
    ifStmt: Vec<IfStmt>,
    whileStmt: Vec<WhileStmt>,
    funcCall: Vec<FuncCall>,
    returnStmt: Vec<ReturnStmt>,
}

impl FuncBody {
    pub fn new(tc: &mut TokenCollection) -> Self {
        let mut assignment = vec!();
        let mut ifStmt = vec!();
        let mut whileStmt = vec!();
        let mut funcCall = vec!();
        let mut returnStmt = vec!();

        while let Some(next_token) = tc.peek_next_token_type() {
            match next_token {
                TokenType::Assignment => {
                    assignment.push(Assignment::new(tc));
                },
                TokenType::IfStatement => {
                    ifStmt.push(IfStmt::new(tc));
                },
                TokenType::WhileStatement => {
                    whileStmt.push(WhileStmt::new(tc));
                },
                TokenType::FuncCall => {
                    funcCall.push(FuncCall::new(tc));

                    match tc.peek_next_token_type() {
                        Some(TokenType::SemiTermination) => {
                            //consume then resume cycle
                            tc.get_next_token();
                            continue;
                        },
                        Some(TokenType::RightBrace) | Some(TokenType::FiStatement) |
                        Some(TokenType::OdStatement) | Some(TokenType::ElseStatement) => {
                            //fall through
                            continue
                        },
                        None => {
                            // Compiler Error :
                            panic!("Expected some form of termination after function call in function body.");
                        },
                        err => {
                            // Compiler Error :
                            panic!("Expected termination sequence after FuncCall, found unexpected Token: {:?}");
                        },
                    }
                },
                TokenType::ReturnStatement => {
                    returnStmt.push(ReturnStmt::new(tc));
                },

                //end of function body sequences
                TokenType::RightBrace | TokenType::FiStatement |
                TokenType::OdStatement | TokenType::ElseStatement => {
                    //consume token? or just return?
                    break
                },

                // Compiler Error :
                err => {
                    panic!("Unable to parse {:?} within function body.", err);
                }
            }
        }

        FuncBody { node_type: TokenType::FuncBody, assignment, ifStmt, whileStmt, funcCall, returnStmt }
    }
}