#![allow(unused_imports,
        non_camel_case_types,
        non_upper_case_globals,
        non_snake_case,
        unused_must_use,
        dead_code,
        unused_doc_comments
)]

/// Std Lib

use std::io::prelude::*;
use std::io::{BufRead, BufReader, Result};
use std::fs::{self, DirEntry};
use std::path::Path;
use std::path::PathBuf;
use std::env;

use std::fmt::Write;
use std::fs::OpenOptions;

/// Internal Lib

mod lib;
use lib::Lexer;
use lib::Lexer::token::{Token,TokenCollection,TokenType};
use lib::Parser;
use lib::IR::ir;
use lib::IR::ir_manager::IRGraphManager;
use lib::IR::ir::{Value,ValTy,Op,InstTy};
use lib::Graph::node::{Node,NodeId};
//use lib::Graph::arena::Arena;
use lib::Utility::display;

/// External Lib

extern crate petgraph;
use petgraph::graph::Graph;
use petgraph::dot::{Dot,Config};

fn main() {
    println!("Hello, Lexer test!");

    //  Some notes for migration later:
    //
    //  -Separate File open into function that returns Result(file, Error)
    //  -separate character reading from interpreatation
    //  -Make this into a command line arguement type program, because how cool would that be?

    //Users/mitcheldickerson/Documents/Projects/Rust/Practice/tiger_compiler/src/Testing
    //once command line, this should not be an issue.

    let mut path = PathBuf::new();
    //env::current_dir().unwrap();
    path.push(env::current_exe().unwrap());
    path.pop(); path.pop(); path.pop(); //this is needed because current .exe is 3 folders too deep.    
    println!("{:?}", path);
    path.push("src/Testing");

    //successfully tested
    //println!("{:?}", path);

    println!("Proof of concept, read each character and print for Tokenization.");

    // Sorts entries based on key
    let mut paths : Vec<_> = fs::read_dir(path.clone()).unwrap()
        .map(|r| r.unwrap())
        .filter(|result| {
            result.file_name().to_str().expect("valid_path_name").contains(".txt")
        })
        .collect();
    paths.sort_by_key(|dir| dir.path());

    for entry in paths
    {
        println!("{:?}", entry);
        println!();

        let mut file = fs::File::open(entry.path()).expect("Error Opening File.");
        let mut token_builder: Vec<lib::Lexer::token::Token> = Vec::new();
        
        let mut buffer = String::new();
        let result = BufReader::new(file).read_to_string(&mut buffer);


        //this works and consumes iter properly!
        let mut char_iter = buffer.chars().peekable();
        let mut read_iter = char_iter.clone();
        
        // #TODO : Need to look up better way to detect an empty iterater.
        //This currently goes way past end of file, but doesn't error, so that is interesting.
        let mut tokens = Lexer::token::TokenCollection::collect(&mut read_iter);
        let mut tc = Lexer::token::TokenCollection::collect(&mut char_iter);

        // Temp remove output to remove clutter.
        //println!("\nTesting Token_Builder results: \n\n{:?}\n\n", tokens.get_vector());

        let comp = Parser::AST::computation::Comp::new(&mut tc);
        let mut irgmanager = comp.to_ir();

        let mut dot_graph_path = entry.file_name();
        let mut file_name = path.to_str().unwrap().to_owned() + "/" + dot_graph_path.to_str().unwrap().trim_end_matches(".txt") + ".dot";

        let mut output = String::new();
        write!(output, "{:?}", display::Dot::with_config(&irgmanager.graph_manager().get_mut_ref_graph().clone(), &[display::Config::EdgeNoLabel]));
        fs::write(file_name, output);
        //write!(file_name, "{:?}", display::Dot::with_config(&irgmanager.get_graph(), &[display::Config::EdgeNoLabel]) as [u8]).expect("File already existed");

        //println!("{:?}", display::Dot::with_config(&irgmanager.get_graph(), &[display::Config::EdgeNoLabel]));

        println!();
        println!();
        println!();

    }
}