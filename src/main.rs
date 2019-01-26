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
    let mut paths : Vec<_> = fs::read_dir(path).unwrap()
        .map(|r| r.unwrap())
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
        comp.to_ir();
        println!();
        println!();
        println!();

    }

    // TODO : Graph tests being done here!
    // TODO : Using standard Graph because it does not require Impl Copy.

    /*
    let mut irm = IRManager::new();

    let mut vec = Node::new(&mut irm);
    vec.get_mut_data_ref().add_instruction(Op::new(Some(Box::new(Value::new(ValTy::con(1)))),None,None,1,0,InstTy::neg));
    vec.get_mut_data_ref().add_instruction(Op::new(Some(Box::new(Value::new(ValTy::con(2)))),Some(Box::new(Value::new(ValTy::con(73)))),None,2,0,InstTy::add));
    vec.get_mut_data_ref().add_instruction(Op::new(Some(Box::new(Value::new(ValTy::con(3)))),None,None,3,0,InstTy::neg));

    let mut vec1 = Node::new(&mut irm);
    vec1.get_mut_data_ref().add_instruction(Op::new(Some(Box::new(Value::new(ValTy::con(12)))),None,None,4,1,InstTy::neg));;
    vec1.get_mut_data_ref().add_instruction(Op::new(Some(Box::new(Value::new(ValTy::con(13)))),None,None,5,1,InstTy::neg));
    vec1.get_mut_data_ref().add_instruction(Op::new(Some(Box::new(Value::new(ValTy::con(14)))),None,None,6,1,InstTy::neg));

    let mut vec2 = Node::new(&mut irm);
    vec2.get_mut_data_ref().add_instruction(Op::new(Some(Box::new(Value::new(ValTy::con(109)))),None,None,7,2,InstTy::neg));;
    vec2.get_mut_data_ref().add_instruction(Op::new(Some(Box::new(Value::new(ValTy::con(108)))),None,None,8,2,InstTy::neg));
    vec2.get_mut_data_ref().add_instruction(Op::new(Some(Box::new(Value::new(ValTy::con(107)))),None,None,9,2,InstTy::neg));

    let mut vec3 = Node::new(&mut irm);
    vec3.get_mut_data_ref().add_instruction(Op::new(Some(Box::new(Value::new(ValTy::con(122)))),None,None,10,3,InstTy::neg));;
    vec3.get_mut_data_ref().add_instruction(Op::new(Some(Box::new(Value::new(ValTy::con(133)))),None,None,11,3,InstTy::neg));
    vec3.get_mut_data_ref().add_instruction(Op::new(Some(Box::new(Value::new(ValTy::con(144)))),None,None,12,3,InstTy::neg));

    let mut og = Graph::new();
    let node1 = og.add_node(vec);
    let node2 = og.add_node(vec1);
    let node3 = og.add_node(vec2);
    let node4 = og.add_node(vec3);

    og.add_edge(node1,node2,1);
    og.add_edge(node1,node3,1);
    og.add_edge(node2,node4,1);
    og.add_edge(node3,node4,1);
    og.add_edge(node4,node1,1);

    println!("{:?}", display::Dot::with_config(&og, &[display::Config::EdgeNoLabel]));
    */

    /*


        let mut b:Vec<Box<dyn lib::IR::ir::Inst>> = vec!();
        b.push(Box::new(ir::Add::new(1, 2)));
        b.push(Box::new(ir::Neg::new(1)));

        for y in b {
            y.debugPrint();
        }


    let mut arena = Arena::new();

    for x in 0..10 {
        let nodeId = arena.new_node();
        arena.get_mut_ref(nodeId.clone()).unwrap().add_instr(Box::new(ir::Add::new(x, x + 1)));
        arena.get_mut_ref(nodeId.clone()).unwrap().add_instr(Box::new(ir::Neg::new(x)));
        arena.get_mut_ref(nodeId).unwrap().add_instr(Box::new(ir::Sub::new(x, x + 1)));
    }

    let mut iter = arena.iter();
    for some in iter {
        for inst in some.instructions() {
            inst.debugPrint();
        }
    }

    let mut node = Node::new(NodeId::new(0));
    node.add_parent(NodeId::new(1));
    node.add_parent(NodeId::new(2));
    node.add_child(NodeId::new(1));
    node.add_child(NodeId::new(3));
    let parents = node.parents();
    let children = node.children();
    println!("{:?}", parents);
    println!("{:?}", children);
    */
}