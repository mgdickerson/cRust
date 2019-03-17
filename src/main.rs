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

    // No longer need to call run, now I can call test and have the failures shown for the specific test (and all still run, how cool!)
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
    path.pop();
    path.pop();
    path.pop(); //this is needed because current .exe is 3 folders too deep.
    println!("{:?}", path);
    path.push("src/Testing");

    //successfully tested
    //println!("{:?}", path);

    println!("Proof of concept, read each character and print for Tokenization.");

    // Sorts entries based on key
    let mut paths: Vec<_> = fs::read_dir(path.clone())
        .unwrap()
        .map(|r| r.unwrap())
        .filter(|result| {
            result
                .file_name()
                .to_str()
                .expect("valid_path_name")
                .contains(".txt")
        })
        .collect();
    paths.sort_by_key(|dir| dir.path());

    for entry in paths {
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

        let mut optimizer = Optimizer::Optimizer::new(irgmanager);
        optimizer.pass_0();
        optimizer.pass_1();
        optimizer.pass_2();
        optimizer.pass_3();
        optimizer.pass_4();

        /* // All of this is now handled by optimizer pass_0
        lib::clean_graph(&mut irgm);

        Optimizer::constant_evaluation::eval_program_constants(&mut irgm);

        let mut temp_val_manager = Optimizer::temp_value_manager::TempValManager::new();
        let main_node = irgm.graph_manager().get_main_node();
        temp_val_manager.pull_temp_values(irgm.graph_manager(), main_node);
        */

        // Getting back irgm from the optimizer.
        let mut main_temp_manager = optimizer.get_main_temp();
        let mut func_temp_manager = optimizer.get_func_temp();
        let mut irgm = optimizer.get_irgm();
        let root_node = irgm.graph_manager().get_main_node();
        let entry_node = irgm.graph_manager().get_main_entrance_node();
        let exit_nodes = irgm.graph_manager().get_exit_nodes(&root_node);

        let mut inst_register_mapping = analyze_live_range(
            &mut irgm,
            &mut main_temp_manager,
            entry_node.clone(),
            exit_nodes,
            None,
            path.clone(),
            entry.file_name().clone(),
        );



        for (func_name, func_root) in irgm.function_manager().list_functions() {
            // Get entry node for function
            let entry_id = irgm
                .graph_manager()
                .get_ref_graph()
                .neighbors_directed(func_root, Incoming)
                .next()
                .unwrap();

            //println!("Analyzing function: {} -> Entry: {:?}", func_name, entry_id);

            let exit_nodes = irgm.graph_manager().get_exit_nodes(&func_root);

            //println!("Exit nodes: {:?}", exit_nodes);

            let func_register_mapping = analyze_live_range(
                &mut irgm,
                &mut func_temp_manager.get_mut(&func_name).unwrap(),
                entry_id,
                exit_nodes,
                Some(func_name),
                path.clone(),
                entry.file_name().clone(),
            );

            for (key, val) in func_register_mapping.iter() {
                if !inst_register_mapping.contains_key(key) {
                    inst_register_mapping.insert(key.clone(), val.clone());
                }
            }
        }

        if inst_register_mapping.len() != 0 {
            phi_absolver::remove_phis(&mut irgm, &mut inst_register_mapping);
            println!("Main has some register mapping.");
        }

        let walkable_graph = irgm.graph_manager_ref().get_ref_graph().clone();
        let mut visited = Vec::new();

        traversal_path(&mut irgm, &walkable_graph, root_node, &mut visited);

        println!("Traversal Path: \n");
        for node_id in visited.iter() {
            println!("{}", irgm.graph_manager_ref().get_ref_graph().node_weight(node_id.clone()).unwrap().get_node_id());
        }

        /*let mut irgm = irgm.clone();

        let root_node = irgm.graph_manager().get_main_node();
        let mut visit_order = irgm.graph_manager().graph_visitor(root_node.clone());
        let final_node = visit_order.pop().expect("Returned a visit order with no nodes in it.");
        let final_node_alias = irgm.graph_manager().get_ref_graph().node_weight(final_node).unwrap().get_node_id();

        println!("Final Node: {:?}", final_node_alias);
        */

        /// TEST SPACE FOR Dominators
        ///
        /// It works!
        let root = irgm.graph_manager().get_main_node();
        let graph = irgm.graph_manager().get_mut_ref_graph().clone();
        let dom_space = simple_fast(&graph, root);
        //println!("{:?}", dom_space);
        for node in graph.node_indices() {
            match dom_space.immediate_dominator(node) {
                Some(parent_node) => {
                    irgm.graph_manager().add_dominance_edge(node, parent_node);
                }
                None => {}
            }
        }

        /// END TEST SPACE ///
        let mut dot_graph_path = entry.file_name();
        let mut file_name = path.to_str().unwrap().to_owned()
            + "/"
            + dot_graph_path.to_str().unwrap().trim_end_matches(".txt")
            + ".dot";

        let mut output = String::new();
        write!(
            output,
            "{:?}",
            display::Dot::with_config(
                &irgm.graph_manager().get_mut_ref_graph().clone(),
                &[display::Config::EdgeColor]
            )
        );
        fs::write(file_name, output);
        //write!(file_name, "{:?}", display::Dot::with_config(&irgm.get_graph(), &[display::Config::EdgeNoLabel]) as [u8]).expect("File already existed");

        //println!("{:?}", display::Dot::with_config(&irgm.get_graph(), &[display::Config::EdgeNoLabel]));

        println!();
        println!();
        println!();
    }
}
