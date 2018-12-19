/*  Go back to modules later after proving concept works
pub mod computation;
pub mod assignment;
*/
use std;
use Lexer::get_token;
use Lexer::token::{Token, TokenType, TokenCollection};

pub mod number;
pub mod ident;
pub mod var;
pub mod array;
<<<<<<< HEAD
=======
pub mod designator;
pub mod expression;
>>>>>>> develop

//  Start by doing a fully recursive parse through the code. Introduce AST later to make
//some kind of storage structure. 


pub fn computation(tc: &mut TokenCollection) {
    //current Token is used for pretty printing and for potential use with AST data structure later.
    //println!("|\t{}", current_token.get_contents());
    //print_ast
    //generateAST()
    while let Some(next_token) = tc.peek_next_token_type() {
        match next_token {
            TokenType::Comment => {
                tc.get_next_token();
            },
            notComment => {
                break;
            }
        }
    }

    match tc.get_next_token().expect("Computation error").get_type() {
        TokenType::Computation => {
            //program does infact start with main. 
            //dont really need to do anything with that
        },
        err => {
            //How in the world did you not get a main token??
            // Compiler Error : 
            panic!("Expecting file to start with keyword 'main', found unexpected Token: {:?}", err);
        },
    }

    //Start by getting next token.
    while let Some(next_token) = tc.peek_next_token_type() {
        match next_token {
            TokenType::Var | TokenType::Array => {
                //found variable declaration
                var_declaration(tc);
            },
            TokenType::FuncDecl => {
                //no variable declaration found, but Function delcaration found
                //drop through
                break;
            },
            TokenType::LeftBrace => {
                //no declarations found
                //drop through
                break;
            },
            err => {
                // Compiler Error : 
                panic!("Expected to find VarDecl, FuncDecl, or Main body start, but found unexpected Token: {:?}", err);
            },
        }
    }

    while let Some(next_token) = tc.peek_next_token_type() {
        match next_token {
            TokenType::FuncDecl => {
                //found funcDecl
                func_declaration(tc);
            },
            TokenType::LeftBrace => {
                //no funcDecl found
                break;
            },
            err => {
                // Compiler Error : 
                panic!("Expected FuncDecl or start of Main body, but found unexpected Token: {:?}", err);
            },
        }
    }

    match tc.peek_next_token_type() {
        Some(TokenType::LeftBrace) => {
            //found body start
            tc.get_next_token();

            func_body(tc);

            //look for closing bracket
            match tc.peek_next_token_type() {
                Some(TokenType::RightBrace) => {
                    tc.get_next_token();
                },
                None => {
                    // Compiler Error : 
                    panic!("Expected '}}' Token at end of main body, found EOF.");
                },
                err => {
                    // Compiler Error : 
                    panic!("Expected '}}' Token at end of main body, found unexpected Token: {:?}", err);
                },
            }
        },
        None => {
            // Compiler Error : 
            panic!("Expected start to main body, found EOF.");
        },
        err => {
            // Compiler Error : 
            panic!("Expected '{{' Token to indicate body start, found unexpected Token: {:?}", err);
        },
    }

    match tc.peek_next_token_type() {
        Some(TokenType::ComputationEnd) => {
            //found end of main computation
            tc.get_next_token();    //consume '.', return
            return
        },
        None => {
            // Compiler Error : 
            panic!("Expected end of main body Token '.', found EOF.");
        },
        err => {
            // Compiler Error : 
            panic!("Expected end of main body Token '.', found unexpected Token: {:?}", err);
        },
    }
}

pub fn var_declaration(tc: &mut TokenCollection) {
    //ast_print
    //generateAST()

    match tc.peek_next_token_type() {
        Some(TokenType::Var) => {
            var(tc);
        },
        Some(TokenType::Array) => {
            array(tc);
        },
        None => {
            // Compiler Error : 
            panic!("Expected variable declaration Var or Array, found EOF.");
        },
        err => {
            // Compiler Error : 
            panic!("Expected Var or Array Token, found unexpected Token: {:?}", err);
        },
    }

    return
}

pub fn func_declaration(tc: &mut TokenCollection) {
    //ast_print
    //generateAST()

    match tc.get_next_token().expect("FuncDecl Error").get_type() {
        TokenType::FuncDecl => {
            //case matches correctly, token is consumed. 
        },
        err => {
            // Compiler Error : 
            panic!("Function delcaration token assumed, but not found. Found : {:?}", err);
        },
    }

    match tc.peek_next_token_type() {
        Some(TokenType::Ident) => {
            func_ident(tc);

            match tc.peek_next_token_type() {
                Some(TokenType::SemiTermination) => {
                    //consume Token then fall through. 
                    tc.get_next_token();
                },
                None => {
                    // Compiler Error : 
                    panic!("Expected ';' at end of function ident, but found EOF.");
                },
                err => {
                    // Compiler Error : 
                    panic!("Expected ';' at end of func_ident, but found unexpected Token: {:?}", err);
                },
            }
        },
        None => {
            // Compiler Error : 
            panic!("Expected Ident Token at function declaration, found EOF.");
        },
        err => {
            // Compiler Error : 
            panic!("Expected Ident Token at function declaration, found unexpected Token: {:?}", err);
        },
    }

    while let Some(next_token) = tc.peek_next_token_type() {
        match next_token {
            TokenType::Var | TokenType::Array => {
                var_declaration(tc);
            },
            TokenType::LeftBrace => {
                //do not consume, fall through
                break;
            },
            err => {
                // Compiler Error : 
                panic!("Expected Variable Decl or '{{' Token for start of function body, but found unexpected Token {:?}", err);
            },
        }
    }

    match tc.peek_next_token_type() {
        Some(TokenType::LeftBrace) => {
            //consume brace, call body
            tc.get_next_token();

            func_body(tc);

            match tc.peek_next_token_type() {
                Some(TokenType::RightBrace) => {
                    //all is well, consume token
                    tc.get_next_token();
                },
                None => {
                    // Compiler Error : 
                    panic!("Expected '}' Token in function body, found EOF.");
                },
                err => {
                    // Compiler Error : 
                    panic!("Expected '}}' Token at end of function body, found unexpected Token: {:?}", err);
                },
            }
        },
        None => {
            // Compiler Error : 
            panic!("Expected either variable declaration or start of function body, found EOF.");
        },
        err => {
            // Compiler Error : 
            panic!("Expected either VarDecl Token or '{{' found unexpected: {:?}", err);
        },
    }

    match tc.peek_next_token_type() {
        Some(TokenType::SemiTermination) => {
            //consume token, return
            tc.get_next_token();
            return
        },
        None => {
            // Compiler Error : 
            panic!("Expected ';' Token at end of function body, found EOF.");
        },
        err => {
            // Compiler Error : 
            panic!("Expected ';' Token at end of function body, found unexpected Token: {:?}", err);
        },
    }
}

pub fn func_ident(tc: &mut TokenCollection) {
    //ast_print
    //generateAST()

    match tc.peek_next_token_type() {
        Some(TokenType::Ident) => {
            //function name
            ident(tc);
        },
        None => {
            // Compiler Error : 
            panic!("Expected Ident Token for function name, found EOF.");
        },
        err => {
            // Compiler Error : 
            panic!("Expected Ident Token for function name, found unexpected Token: {:?}", err);
        },
    }

    match tc.peek_next_token_type() {
        Some(TokenType::LeftBrace) => {
            //function parameter start 
            tc.get_next_token();
            
            func_param(tc);

            match tc.peek_next_token_type() {
                Some(TokenType::RightBrace) => {
                    tc.get_next_token();
                },
                Some(TokenType::SemiTermination) => {
                    //pass through to return statement
                },
                None => {
                    // Compiler Error : 
                    panic!("Expected ')' Token, found EOF.");
                },
                err => {
                    // Compiler Error : 
                    panic!("Expected ')' Token at end of function parameters, found unexpected Token: {:?}", err);
                },
            }
        },
        Some(TokenType::SemiTermination) => {
            //no parameters to pass, fall through
        },
        None => {
            // Compiler Error : 
            panic!("Expected '(' Token, found EOF.");
        },
        err => {
            // Compiler Error : 
            panic!("Expected '(' Token at end of function parameters, found unexpected Token: {:?}", err);
        },
    }

    return

    //each individual func_ident command will handle its own possible semi-colons
    /*
    match tc.peek_next_token_type() {
        Some(TokenType::SemiTermination) => {
            //do not consume ';' Token because we need generic behavior for func_call
            //tc.get_next_token();
            return
        },
        None => {
            // Compiler Error : 
            panic!("Expected ';' Token, found EOF.");
        },
        err => {
            // Compiler Error : 
            panic!("Expected ';' Token at end of function parameters, found unexpected Token: {:?}", err);
        },
    }
    */
}

pub fn func_param(tc: &mut TokenCollection) {
    //ast_print
    //generateAST()

    while let Some(next_token) = tc.peek_next_token_type() {
        match next_token {
            TokenType::Ident => {
                //get parameter ident
                ident(tc);
            },
            TokenType::Comma => {
                //consume token
                tc.get_next_token();
                match tc.peek_next_token_type() {
                    Some(TokenType::Ident) => {
                        //all is well, drop through
                        continue;
                    },
                    None => {
                        // Compiler Error : 
                        panic!("Unexpected EOF, expected Ident Token following ',' in function param.");
                    },
                    err => {
                        // Compiler Error : 
                        panic!("Expected Ident Token following ',', found unexpected Token: {:?}", err);
                    },
                }
            },
            TokenType::RightBrace => {
                //end of function, return to func_ident but do not consume token
                return 
            },
            err => {
                // Compiler Error : 
                panic!("Expected Function Parameters, was unable to parse: {:?}", err);
            },
        }
    }
}

pub fn func_body(tc: &mut TokenCollection) {
    //ast_print
    //generateAST()

    while let Some(next_token) = tc.peek_next_token_type() {
        match next_token {
            TokenType::Assignment => {
                assignment(tc);
            },
            TokenType::IfStatement => {
                if_stat(tc);
            },
            TokenType::WhileStatement => {
                while_stat(tc);
            },
            TokenType::FuncCall => {
                func_call(tc);

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
                rtn_stat(tc);
            },
            
            //end of function body sequences
            TokenType::RightBrace | TokenType::FiStatement | 
            TokenType::OdStatement | TokenType::ElseStatement => {
                //consume token? or just return?
                return
            },

            // Compiler Error : 
            err => {
                panic!("Unable to parse {:?} within function body.", err);
            }
        }
    }

}

pub fn assignment(tc: &mut TokenCollection) {
    //ast_print
    //generateAST()

    match tc.get_next_token().expect("Assignment Error").get_type() {
        TokenType::Assignment => {
            //expected assignment token found. 
        },
        err => {
            // Compiler Error : 
            panic!("Expected to find Assignment token, found unexpected Token: {:?}", err);
        },
    }

    match tc.peek_next_token_type() {
        Some(TokenType::Ident) => {
            designator(tc);
        },
        err => {
            // Compiler Error : 
            panic!("Expected Designator for assignment variable, found unexpected Token: {:?}", err);
        },
    }

    match tc.get_next_token().expect("Assignment Op Error").get_type() {
        TokenType::AssignmentOp => {
            //expected assignment operator found, proceed to expression.
            expression(tc);
        },
        err => {
            // Compiler Error : 
            panic!("Expected Assignment Operator '<-', found unexpected Token: {:?}", err);
        },
    }

    match tc.peek_next_token_type() {
        Some(TokenType::SemiTermination) => {
            //consume token, return. 
            tc.get_next_token();
            return
        },
        // All Possible Ending Sequences where ';' may not be necessary. 
        Some(TokenType::FiStatement) | Some(TokenType::OdStatement) | 
        Some(TokenType::RightBrace) | Some(TokenType::ElseStatement) => {
            //';' not required, return without consuming token. 
            return
        },
        None => {
            // Compiler Error : 
            panic!("Expected end of assignment, found EOF.");
        },
        err => {
            // Compiler Error : 
            panic!("Expected end of assignment, found unexpected Token: {:?}", err);
        },
    }
}

pub fn if_stat(tc: &mut TokenCollection) {
    //ast_print
    //generateAST()

    match tc.get_next_token().expect("If Statment Error").get_type() {
        TokenType::IfStatement => {
            //expected if statement token found
            //Next statement should be a relation type expression
            relation(tc);
        },
        err => {
            // Compiler Error : 
            panic!("Expected If Statement, found unexpected Token: {:?}", err);
        },
    }

    match tc.peek_next_token_type() {
        Some(TokenType::ThenStatement) => {
            //Found Then token, consume token and move forward. 
            tc.get_next_token();
            func_body(tc);
        }
        None => {
            // Compiler Error : 
            panic!("Unexpected end of file after if relation.");
        },
        err => {
            // Compiler Error : 
            panic!("Expected Then token, found unexpected Token: {:?}", err);
        },
    }

    match tc.peek_next_token_type() {
        Some(TokenType::ElseStatement) => {
            //consume the else, pass body of statement
            tc.get_next_token();
            func_body(tc);
        },
        Some(TokenType::FiStatement) => {
            //fall through to next match case. this is just an else handler. 
        },
        None => {
            // Compiler Error : 
            panic!("Unexpected end of file. Expected Else or fi statement.");
        },
        err => {
            // Compiler Error : 
            panic!("Expected Else or fi statment, found unexpected Token: {:?}", err);
        },
    }

    match tc.peek_next_token_type() {
        Some(TokenType::FiStatement) => {
            tc.get_next_token();
            match tc.peek_next_token_type() {
                Some(TokenType::SemiTermination) => {
                    //consume token, return. 
                    tc.get_next_token();
                    return
                },
                // All Possible Ending Sequences where ';' may not be necessary. 
                Some(TokenType::FiStatement) | Some(TokenType::OdStatement) | 
                Some(TokenType::RightBrace) | Some(TokenType::ElseStatement) => {
                    //';' not required, return without consuming token. 
                    return
                },
                None => {
                    // Compiler Error : 
                    panic!("Expected Fi statement, none was found.");
                },
                err => {
                    // Compiler Error : 
                    panic!("Expected Fi statement, found unexpected Token: {:?}", err);
                },
            }
        },
        err => {
            // Compiler Error : 
            panic!("Expected Else or fi statment, found unexpected Token: {:?}", err);
        }
    }
}

pub fn while_stat(tc: &mut TokenCollection) {
    //ast_print
    //generateAST()

    match tc.get_next_token().expect("While Statement Error").get_type() {
        TokenType::WhileStatement => {
            //expected token was found, next do relation
            relation(tc);
        },
        err => {
            // Compiler Error : 
            panic!("Expected While statement, found unexpected Token: {:?}", err);
        },
    }

    match tc.peek_next_token_type() {
        Some(TokenType::DoStatement) => {
            tc.get_next_token();
            func_body(tc);
        },
        None => {
            // Compiler Error : 
            panic!("Unexpected End of File, expected do statement.");
        },
        err => {
            // Compiler Error : 
            panic!("Expected do statement, found unexpected Token: {:?}", err);
        },
    }

    match tc.peek_next_token_type() {
        Some(TokenType::OdStatement) => {
            tc.get_next_token();
            match tc.peek_next_token_type() {
                Some(TokenType::SemiTermination) => {
                    //Found ';' so there are likely to be more statements. Consume and return. 
                    tc.get_next_token();
                    return
                },
                // All Possible Ending Sequences where ';' may not be necessary. 
                Some(TokenType::FiStatement) | Some(TokenType::OdStatement) | 
                Some(TokenType::RightBrace) | Some(TokenType::ElseStatement) => {
                    //';' not required, return without consuming token. 
                    return
                },
                None => {
                    // Compiler Error : 
                    panic!("End of file found, do should be appended by '}' if end of statement");
                },
                err => {
                    // Compiler Error : 
                    panic!("Expected to find ';' or end of block after Od statement, found unexpected Token: {:?}", err);
                },
            }
        },
        None => {
            // Compiler Error : 
            panic!("Unexpected End of File, expected Od Token.");
        },
        err => {
            // Compiler Error : 
            panic!("Expected Od Token, found unexpected Token: {:?}", err);
        },
    }
}

pub fn func_call(tc: &mut TokenCollection) {
    //ast_print
    //generateAST()

    match tc.get_next_token().expect("Func Call Error").get_type() {
        TokenType::FuncCall => {
            //this is as was expected, call function ident;
            
            //check for function identity. 
            match tc.peek_next_token_type() {
                Some(TokenType::Ident) => {
                    ident(tc);   //this will grab function identity AND parameters.

                    match tc.peek_next_token_type() {
                        Some(TokenType::LeftBrace) => {
                            //function parameter start 
                            tc.get_next_token();
                            
                            while let Some(next_token) = tc.peek_next_token_type() {
                                match next_token {
                                    TokenType::RightBrace => {
                                        tc.get_next_token();
                                        break;
                                    },
                                    TokenType::Comma => {
                                        //consume token and get next expr
                                        tc.get_next_token();
                                        expression(tc);
                                    },
                                    expr => {
                                        //get next expression
                                        expression(tc);
                                    },
                                }
                            }
                        },
                        None => {
                            // Compiler Error : 
                            panic!("Expected anything after func_call, found EOF.");
                        },
                        ret => {
                            //it was literally any other case, so just return and handle elsewhere
                            return
                        },
                    }
                    // TODO : 
                    //here we add to some table with function declaration and function call.
                    //depending on the table, we could declare a function after it is used similar to rust.
                    //this would require meta-data and unwinding possible errors in functions
                    //not existing. 
                },
                None => {
                    // Compiler Error : 
                    panic!("Expected Function Identity, found end of file.");
                },
                err => {
                    // Compiler Error : 
                    panic!("Expected Function Identity, found unexpected Token: {:?}", err);
                },
            }
        },
        err => {
            // Compiler Error : 
            panic!("Expected Functional Call Token, found unexpected Token: {:?}", err);
        },
    }

    //can probably just return after this
    return

    /*
    match tc.peek_next_token_type() {
        Some(TokenType::SemiTermination) => {
            return
        },
        // All Possible Ending Sequences where ';' may not be necessary. 
        Some(TokenType::FiStatement) | Some(TokenType::OdStatement) | 
        Some(TokenType::RightBrace) | Some(TokenType::MathOp) | 
        Some(TokenType::MulOp) => {
            //';' not required, return without consuming token. 
            return
        },
        None => {
            // Compiler Error : 
            panic!("End of file found, do should be appended by '}' if end of statement");
        },
        _ => {
            // Compiler Error : 
            panic!("Unexpected Token, expected to find ';' or end of block after Od statement.");
        },
    }
    */
}

pub fn rtn_stat(tc: &mut TokenCollection) {
    //ast_print
    //generateAST()

    match tc.get_next_token().expect("Return Statement Error").get_type() {
        TokenType::ReturnStatement => {
            // return token found, process the expression
            expression(tc);
        },
        // TODO : fix up to proper error handler
        err => { println!("Expected Return Statement, found unexpected Token: {:?}", err); },  //proper method of error handling unexpected tokens
    }

    match tc.peek_next_token_type() {
        Some(TokenType::SemiTermination) => {
            //Found ';' so there are likely to be more statements. Consume and return. 
            tc.get_next_token();
            return
        },
        // All Possible Ending Sequences where ';' may not be necessary. 
        Some(TokenType::FiStatement) | Some(TokenType::OdStatement) | 
        Some(TokenType::RightBrace) | Some(TokenType::ElseStatement) => {
            //';' not required, return without consuming token. 
            return
        },
        None => {
            // Compiler Error : 
            panic!("End of file found, do should be appended by '}' if end of statement");
        },
        err => {
            // Compiler Error : 
            panic!("Expected to find ';' or end  statement, found unexpected Token: {:?}", err);
        },
    }
}

pub fn relation(tc: &mut TokenCollection) {
    //ast_print
    //generateAST()

    //evaluate first expression;
    expression(tc);

    match tc.peek_next_token_type() {
        Some(TokenType::RelOp) => {
            //consume token
            tc.get_next_token();
        },
        None => {
            // Compiler Error : 
            panic!("Expected RelOp token, found EOF.");
        },
        err => {
            // Compiler Error : 
            panic!("Expected RelOp token, unexpected Token {:?} was found instead.", err);
        },
    }

    expression(tc);

    //relation is built, return
    return
}

pub fn expression(tc: &mut TokenCollection) {
    //ast_print
    //generateAST()

    term(tc);

    loop {
        //handle MathOp possibility
        match tc.peek_next_token_type() {
            Some(TokenType::MathOp) => {
                //MathOp found, call another term. 
                tc.get_next_token();    // consume the MathOp
                term(tc);
            },
            None => {
                // Compiler Error : 
                panic!("Unexpected EOF in expression.");
            },
            _ => {
                //If there is no MathOp, return. Dont do any other debugging or logic here. 
                return
            },
        }
    }
}

pub fn term(tc: &mut TokenCollection) {
    //ast_print
    //generateAST()

    factor(tc);

    loop {
        //handle MulOp possibility
        match tc.peek_next_token_type() {
            Some(TokenType::MulOp) => {
                //MulOp found, consume then call factor again
                tc.get_next_token();
                factor(tc);
            },
            None => {
                // Compiler Error : 
                panic!("Unexpected EOF in term.");
            },
            _ => {
                //If no MulOp, return
                return
            },
        }
    }
}

pub fn factor(tc: &mut TokenCollection) {
    //ast_print
    //generateAST()

    match tc.peek_next_token_type() {
        Some(TokenType::Ident) => {
            designator(tc);
        },
        Some(TokenType::Number) => {
            number(tc);
        },
        Some(TokenType::FuncCall) => {
            func_call(tc);
        },
        Some(TokenType::LeftBrace) => {
            //consume token, call self
            tc.get_next_token();
            expression(tc);

            //handle closing brace in initial call of brace so all braces ar self contained. 
            match tc.peek_next_token_type() {
                Some(TokenType::RightBrace) => {
                    tc.get_next_token();
                    //fall through
                },
                None => {
                    // Compiler Error : 
                    panic!("Expected Closing ')' Token for expression, found EOF.");
                },
                err => {
                    // Compiler Error : 
                    panic!("Expected Closing ')' Token for expression, found unexpected Token: {:?}", err);
                },
            }
        },
        None => {
            // Compiler Error : 
            panic!("Expected Designator, Number, Function call, or '(' Token, found EOF.");
        },
        err => {
            // Compiler Error : 
            panic!("Expected Designator, Number, Function Call, or '(' Token, found unexpected {:?}", err);
        },
    }

    return
}

pub fn designator(tc: &mut TokenCollection) {
    //ast_print
    //generateAST()
    
    match tc.peek_next_token_type() {
        Some(TokenType::Ident) => {
            ident(tc);

            while let Some(next_token) = tc.peek_next_token_type() {
                match next_token {
                    TokenType::LeftBrace => {
                        //consume left brace
                        tc.get_next_token();

                        expression(tc);

                        //consume next token if right brace
                        match tc.peek_next_token_type() {
                            Some(TokenType::RightBrace) => {
                                //consume right brace
                                tc.get_next_token();
                            },
                            None => {
                                // Compiler Error : 
                                panic!("Unexpected EOF, expected ']' token for designator.");
                            },
                            err => {
                                // Compiler Error : 
                                panic!("Unexpected Token: {:?}, expected ']' token for designator.", err);
                            },
                        }
                    },
                    _ => {
                        //ident already collected, bail. no need for error handling here. 
                        return
                    },
                }
            }
        }

        None => {
            // Compiler Error : 
            panic!("Unexpected EOF, expected Ident token for designator.");
        },
        err => {
            // Compiler Error : 
            panic!("Expected Ident Token in designator, found unexpected Token: {:?}", err);
        },
    }
}

pub fn array(tc: &mut TokenCollection) {
    //ast_print
    //generateAST()

    match tc.get_next_token().expect("Array Error").get_type() {
        TokenType::Array => {
            // proper action, all is well.
        },
        err => {
            // Compiler Error : 
            panic!("Expected Array declaration, found unexpected Token: {:?}", err);
        },
    }

    match tc.peek_next_token_type() {
        Some(TokenType::LeftBrace) => {
            //all is well, proceed through
        },
        None => {
            // Compiler Error : 
            panic!("Expected Array '[' Token, found EOF.");
        },
        err => {
            // Compiler Error : 
            panic!("Expected '[' Token in array declaration, found unexpected Token: {:?}", err);
        },
    }

    while let Some(next_token) = tc.peek_next_token_type() {
        match next_token {
            TokenType::LeftBrace => {
                //should this update depth of array? Or do these just get consumed?
                //for now, just consume. 
                // TODO : Confirm it is the right bracket type. '['
                tc.get_next_token();

                match tc.peek_next_token_type() {
                    Some(TokenType::Number) => {
                        number(tc);
                    },
                    None => {
                        // Compiler Error : 
                        panic!("Expected Array index Number, found EOF.");
                    },
                    err => {
                        // Compiler Error : 
                        panic!("Expected Number Token in array index, found unexpected Token: {:?}", err);
                    },
                }

                match tc.peek_next_token_type() {
                    Some(TokenType::RightBrace) => {
                        tc.get_next_token();
                    },
                    None => {
                        // Compiler Error : 
                        panic!("Expected Array ']', found EOF.");
                    },
                    err => {
                        // Compiler Error : 
                        panic!("Expected ']' Token in array declaration, found unexpected Token: {:?}", err);
                    },
                }
            },
            TokenType::Ident => {
                // first ident case found, break from loop. 
                break;
            }
            err => {
                // Compiler Error : 
                panic!("Expected Array T: {:?}", err);
            },
        }
    }

    while let Some(next_token) = tc.peek_next_token_type() {
        match next_token {
            TokenType::Ident => {
                ident(tc);
            },
            TokenType::Comma => {
                tc.get_next_token();
                continue;
            }
            TokenType::SemiTermination => {
                tc.get_next_token();
                return
            }
            err => {
                // Compiler Error : 
                panic!("Expected Ident Token in array declaration, found unexpected Token: {:?}", err);
            },
        }
    }


}
//---------------------------------//

pub fn var(tc: &mut TokenCollection) {
    //print_ast
    //generateAST()
    match tc.get_next_token().expect("Var Error").get_type() {
        TokenType::Var => {
            //This is accepted behavior, pass through. 
        },
        err => {
            // Compiler Error : 
            panic!("Expected Variable declaration, found unexpected Token: {:?}", err);
        }
    }

    while let Some(next_token) = tc.peek_next_token_type() {
        match next_token {
            TokenType::Ident => {
                ident(tc);
            },
            TokenType::Comma => {
                //consume comma token
                tc.get_next_token();
            },
            TokenType::SemiTermination => {
                //consume semicolon and return. 
                tc.get_next_token();
                return
            },
            err => {
                // Compiler Error : 
                panic!("Unable to parse token in variable declaration: {:?}", err);
            },
        }
    }
}

pub fn ident(tc: &mut TokenCollection) {
    //print_ast
    //generateAST()

    tc.get_next_token();
    //should return some AST data structure
}


pub fn number(tc: &mut TokenCollection) {
    //print_ast
    //generateAST()

    tc.get_next_token();
    //should return some AST data structure
}
