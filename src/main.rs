#![allow(
    unused_imports,
    non_camel_case_types,
    non_upper_case_globals,
    non_snake_case,
    unused_must_use,
    dead_code,
    unused_doc_comments
)]

use std::env;
use std::fs::{self, DirEntry};
/// Std Lib
use std::io::prelude::*;
use std::io::{BufRead, BufReader, Result};
use std::path::Path;
use std::path::PathBuf;

use std::fmt::Write;
use std::fs::OpenOptions;

/// Internal Lib
mod lib;
use lib::Graph::node::{Node, NodeId};
use lib::Lexer;
use lib::Lexer::token::{Token, TokenCollection, TokenType};
use lib::Optimizer;
use lib::Parser;
use lib::RegisterAllocator::analyze_live_range;
use lib::Utility::display;
use lib::IR::ir;
use lib::IR::ir::{InstTy, Op, ValTy, Value};
use lib::IR::ir_manager::IRGraphManager;
use lib::CodeGen::{phi_absolver,generate_code::traversal_path};

/// External Lib
extern crate petgraph;

use petgraph::algo::dominators::simple_fast;
use petgraph::algo::dominators::Dominators;

use petgraph::algo::DfsSpace;
use petgraph::dot::{Config, Dot};
use petgraph::prelude::NodeIndex;
use petgraph::visit::Dfs;
use petgraph::Graph;
use petgraph::{Directed, Incoming, Outgoing};

fn main() {
    // TODO : Start building command line tool.

    let args: Vec<String> = env::args().collect();
    let mut path = args[1].clone();
    if path.starts_with("./") {
        path = env::current_dir().unwrap().to_str().unwrap().to_owned() + path.trim_left_matches(".");
    }

    //println!("{:?}", path);
    lib::run(path);
}
