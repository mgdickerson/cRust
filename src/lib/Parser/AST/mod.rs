use std;
use Lexer::get_token;
use Lexer::token::{Token, TokenType, TokenCollection};

use lib::Graph::node::{Node, NodeId, NodeType, NodeData};

extern crate petgraph;
use petgraph::graph::Graph;

use lib::IR::ir_manager::IRGraphManager;
use lib::IR::ir::{Value,ValTy,Op,InstTy};

pub mod number;
pub mod ident;
pub mod var;
pub mod array;
pub mod designator;
pub mod expression;
pub mod factor;
pub mod term;
pub mod relation;
pub mod return_stmt;
pub mod func_call;
pub mod while_stmt;
pub mod if_stmt;
pub mod assignment;
pub mod func_body;
pub mod func_param;
pub mod func_ident;
pub mod func_decl;
pub mod var_decl;
pub mod computation;