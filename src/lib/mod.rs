use std::io::{BufRead, BufReader, Result};
use std::fs::{self, DirEntry};
use std::path::Path;
use std::path::PathBuf;
use std::env;
use std::io::prelude::*;
use std::fmt::Write;
use std::fs::OpenOptions;

use self::Utility::display;

extern crate petgraph;
use petgraph::graph;

use petgraph::algo::dominators::Dominators;
use petgraph::algo::dominators::simple_fast;
use lib::IR::ir_manager::IRGraphManager;

pub mod Lexer;
pub mod Parser;
pub mod Utility;
pub mod IR;
pub mod Graph;
pub mod Optimizer;

#[cfg(test)]
pub mod tests {
    use lib::run_file;

    #[test]
    fn test_big() {
        run_file(String::from("big"));
    }

    #[test]
    fn test_cell() {
        run_file(String::from("cell"));
    }

    #[test]
    fn test_conditional_call() {
        run_file(String::from("conditional_call"));
    }

    #[test]
    fn test_factorial() {
        run_file(String::from("factorial"));
    }

    #[test]
    fn test_op_dom_test() {
        run_file(String::from("op_dom_test"));
    }

    #[test]
    fn test_simple_reassignment() {
        run_file(String::from("simple_reassignment"));
    }

    #[test]
    fn test_001() {
        run_file(String::from("test001"));
    }

    #[test]
    fn test_001_a() {
        run_file(String::from("test001_a"));
    }

    #[test]
    fn test_002() {
        run_file(String::from("test002"));
    }
    #[test]
    fn test_003() {
        run_file(String::from("test003"));
    }
    #[test]
    fn test_004() {
        run_file(String::from("test004"));
    }
    #[test]
    fn test_004_a() {
        run_file(String::from("test004_a"));
    }
    #[test]
    fn test_005() {
        run_file(String::from("test005"));
    }
    #[test]
    fn test_006() {
        run_file(String::from("test006"));
    }
    #[test]
    fn test_007() {
        run_file(String::from("test007"));
    }
    #[test]
    fn test_008() {
        run_file(String::from("test008"));
    }
    #[test]
    fn test_009() {
        run_file(String::from("test009"));
    }
    #[test]
    fn test_010() {
        run_file(String::from("test010"));
    }
    #[test]
    fn test_011() {
        run_file(String::from("test011"));
    }
    #[test]
    fn test_012() {
        run_file(String::from("test012"));
    }
    #[test]
    fn test_013() {
        run_file(String::from("test013"));
    }
    #[test]
    fn test_014() {
        run_file(String::from("test014"));
    }
    #[test]
    fn test_015() {
        run_file(String::from("test015"));
    }
    #[test]
    fn test_016() {
        run_file(String::from("test016"));
    }
    #[test]
    fn test_016_a() {
        run_file(String::from("test016_a"));
    }
    #[test]
    fn test_017() {
        run_file(String::from("test017"));
    }
    #[test]
    fn test_018() {
        run_file(String::from("test018"));
    }
    #[test]
    fn test_019() {
        run_file(String::from("test019"));
    }
    #[test]
    fn test_020() {
        run_file(String::from("test020"));
    }
    #[test]
    fn test_021() {
        run_file(String::from("test021"));
    }
    #[test]
    fn test_022() {
        run_file(String::from("test022"));
    }
    #[test]
    fn test_023() {
        run_file(String::from("test023"));
    }
    #[test]
    fn test_024() {
        run_file(String::from("test024"));
    }
    #[test]
    fn test_024_a() {
        run_file(String::from("test024_a"));
    }
    #[test]
    fn test_024_b() {
        run_file(String::from("test024_b"));
    }
    #[test]
    fn test_025() {
        run_file(String::from("test025"));
    }
    #[test]
    fn test_026() {
        run_file(String::from("test026"));
    }
    #[test]
    fn test_027() {
        run_file(String::from("test027"));
    }
    #[test]
    fn test_028() {
        run_file(String::from("test028"));
    }
    #[test]
    fn test_029() {
        run_file(String::from("test029"));
    }
    #[test]
    fn test_030() {
        run_file(String::from("test030"));
    }
    #[test]
    fn test_031() {
        run_file(String::from("test031"));
    }
}

pub fn run_file(file_name: String) {
    let mut path = PathBuf::new();
    path.push(env::current_exe().unwrap());
    path.pop(); path.pop(); path.pop(); path.pop(); //this is needed because current .exe is 4 folders too deep.
    path.push("src/Testing/");
    let mut base_path = path.clone();
    path.push(file_name.clone() + ".txt");
    println!("{:?}", path);



    let mut file = fs::File::open(path.as_path()).expect("Error Opening File.");
    let mut token_builder: Vec<Lexer::token::Token> = Vec::new();

    let mut buffer = String::new();
    let result = BufReader::new(file).read_to_string(&mut buffer);

    let mut char_iter = buffer.chars().peekable();
    let mut read_iter = char_iter.clone();

    let mut tokens = Lexer::token::TokenCollection::collect(&mut read_iter);
    let mut tc = Lexer::token::TokenCollection::collect(&mut char_iter);

    let comp = Parser::AST::computation::Comp::new(&mut tc);
    let mut irgmanager = comp.to_ir();

    clean_graph(&mut irgmanager);

    Optimizer::constant_evaluation::eval_program_constants(&mut irgmanager);

    /// TEST SPACE FOR Dominators
    ///
    /// It works!

    let root = irgmanager.graph_manager().get_main_node();
    let graph = irgmanager.graph_manager().get_mut_ref_graph().clone();
    let dom_space = simple_fast(&graph,root);
    //println!("{:?}", dom_space);
    for node in graph.node_indices() {
        match dom_space.immediate_dominator(node) {
            Some(parent_node) => {
                irgmanager.graph_manager().add_dominance_edge(node, parent_node);
            },
            None => {},
        }
    }

    /// END TEST SPACE ///

    base_path.push(file_name + ".dot");

    let mut output = String::new();
    write!(output, "{:?}", display::Dot::with_config(&irgmanager.graph_manager().get_mut_ref_graph().clone(), &[display::Config::EdgeColor]));
    fs::write(base_path.as_path(), output);

    println!();
    println!();
    println!();
}

pub fn clean_graph(irgm: &mut IRGraphManager) {
    for node in irgm.graph_manager().get_mut_ref_graph().node_weights_mut() {
        for inst in node.get_mut_data_ref().get_inst_list_ref() {
            inst.borrow_mut().update_base_values();
        }
    }
}