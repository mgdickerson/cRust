use std::io::prelude::*;
use std::io::{BufRead, BufReader, Result};
use std::fs::{self, DirEntry};
use std::path::Path;
use std::path::PathBuf;
use std::env;

mod lib;
use lib::Lexer;
use lib::Lexer::token::{Token,TokenCollection,TokenType};
use lib::Parser;
use lib::IR::ir;
use lib::IR::ir::{Operand,OpType};
use lib::Graph::node::{Node,NodeId};
//use lib::Graph::arena::Arena;

use typed_arena::Arena;

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

    for entry in fs::read_dir(path).unwrap()
    {
        println!("{:?}", entry);

        let mut file = fs::File::open(entry.unwrap().path()).expect("Error Opening File.");
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
        
        println!("\nTesting Token_Builder results: \n\n{:?}\n\n", tokens.get_vector());

        let comp = Parser::AST::computation::Comp::new(&mut tc);
    }

    // TODO : Graph tests being done here!

    let monster = Arena::new();
    let test = monster.alloc(Node::new(NodeId::new(0)));

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