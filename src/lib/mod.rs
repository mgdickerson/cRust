use std::cell::RefCell;
use std::env;
use std::fmt::Write;
use std::fs::OpenOptions;
use std::fs::{self, DirEntry};
use std::io::prelude::*;
use std::io::{BufRead, BufReader, Result};
use std::path::Path;
use std::path::PathBuf;
use std::rc::Rc;

use self::Utility::display;

extern crate petgraph;
use petgraph::graph;
use petgraph::{Directed, Incoming, Outgoing};
use petgraph::algo::dominators::simple_fast;
use petgraph::algo::dominators::Dominators;
use petgraph::prelude::NodeIndex;

use lib::Optimizer::temp_value_manager::TempValManager;
use lib::RegisterAllocator::analyze_live_range;
use lib::RegisterAllocator::{Color, RegisterAllocation};
use lib::IR::ir::{InstTy, ValTy, Value};
use lib::IR::ir_manager::IRGraphManager;
use lib::Graph::node::Node;

use lib::Lexer::token::TokenCollection;
use lib::Parser::AST::computation::Comp;
use lib::CodeGen::{phi_absolver,generate_code,instruction_builder};
use std::collections::HashMap;

pub mod CodeGen;
pub mod Graph;
pub mod IR;
pub mod Lexer;
pub mod Optimizer;
pub mod Parser;
pub mod RegisterAllocator;
pub mod Utility;

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
    fn test_025_a() {
        run_file(String::from("test025_a"));
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

pub fn run(path_name: String) {
    // Take a file name provided by main, perform all passes on it.
    let mut path = PathBuf::new();
    path.push(path_name.clone());

    let file = fs::File::open(path.as_path())
        .expect("Unable to find file. Perhaps directory was entered incorrectly?");

    let irgman = parse(file);
    let (mut irgm, mut mtm, mut ftm) =
        optimize_passes(irgman, path.clone());
    let mut inst_register_mapping = register_allocation(&mut irgm, &mut mtm, &mut ftm, path.clone());

    if inst_register_mapping.len() != 0 {
        phi_absolver::remove_phis(&mut irgm, &mut inst_register_mapping);
    }

    let mut code_gen = generate_code::CodeGen::new(irgm, inst_register_mapping);
    code_gen.clean_inst();

    // TODO : Add in everything after register allocation (Otherwise this works! probably want to make the parser for the command line more robust)
    let mut irgm = code_gen.get_irgm();
    add_dominance_path(&mut irgm, path.clone());
}

fn tokenize(source: std::fs::File) -> TokenCollection {
    let mut buffer = String::new();
    let result = BufReader::new(source)
        .read_to_string(&mut buffer);

    match result {
        Ok(num) => {
            let mut char_iter = buffer.chars().peekable();
            TokenCollection::collect(&mut char_iter)
        },
        Err(e) => {
            panic!(e);
        }
    }
}

fn parse(source: std::fs::File) -> IRGraphManager {
    // Gather all tokens
    let mut tc = tokenize(source);

    // Feed tokens into parser, return IRGraphManager
    Comp::new(&mut tc).to_ir()
}

fn optimize_passes(irgm: IRGraphManager, path_buf: PathBuf)
    -> (IRGraphManager, TempValManager, HashMap<String,TempValManager>) {
    let mut optimizer = Optimizer::Optimizer::new(irgm);

    // Output graph after every optimization pass.
    optimizer.pass_0();
    print_graph(
        path_buf.to_str().unwrap().clone()
            .trim_end_matches(".txt").to_owned() + "_opt0.dot",
        optimizer.get_irgm_ref().graph_manager_ref().get_ref_graph()
    );

    optimizer.pass_1();
    print_graph(
        path_buf.to_str().unwrap().clone()
            .trim_end_matches(".txt").to_owned() + "_opt1.dot",
        optimizer.get_irgm_ref().graph_manager_ref().get_ref_graph()
    );

    optimizer.pass_2();
    print_graph(
        path_buf.to_str().unwrap().clone()
            .trim_end_matches(".txt").to_owned() + "_opt2.dot",
        optimizer.get_irgm_ref().graph_manager_ref().get_ref_graph()
    );

    optimizer.pass_3();
    print_graph(
        path_buf.to_str().unwrap().clone()
            .trim_end_matches(".txt").to_owned() + "_opt3.dot",
        optimizer.get_irgm_ref().graph_manager_ref().get_ref_graph()
    );

    optimizer.pass_4();
    print_graph(
        path_buf.to_str().unwrap().clone()
            .trim_end_matches(".txt").to_owned() + "_opt4.dot",
        optimizer.get_irgm_ref().graph_manager_ref().get_ref_graph()
    );

    let main_temp = optimizer.get_main_temp();
    let func_temps = optimizer.get_func_temp();

    (optimizer.get_irgm(), main_temp, func_temps)
}

fn print_graph(path: String, graph: &petgraph::graph::Graph<Node, String, Directed, u32>) {
    let mut output = String::new();
    write!(output, "{:?}", display::Dot::with_config(
        graph,
        &[display::Config::EdgeColor]
    ));
    fs::write(path, output);
}

fn register_allocation(irgm: &mut IRGraphManager,
                       mtm: &mut TempValManager,
                       ftm: &mut HashMap<String,TempValManager>,
                       path: PathBuf
) -> HashMap<usize, usize> {
    let root_node = irgm.graph_manager_ref().get_main_node();
    let entry_node = irgm.graph_manager_ref().get_main_entrance_node();
    let exit_nodes = irgm.graph_manager_ref().get_exit_nodes(&root_node);

    let mut inst_register_mapping = analyze_live_range(
        irgm,
        mtm,
        root_node,
        exit_nodes,
        None,
        path.clone()
    );

    for (func_name, func_root) in irgm.function_manager().list_functions() {
        let entry_id = irgm
            .graph_manager_ref()
            .get_ref_graph()
            .neighbors_directed(func_root, Incoming)
            .next()
            .unwrap();

        let exit_nodes = irgm.graph_manager().get_exit_nodes(&func_root);

        let func_register_mapping = analyze_live_range(
            irgm,
            ftm.get_mut(&func_name).unwrap(),
            entry_id,
            exit_nodes,
            Some(func_name),
            path.clone()
        );

        for (key, value) in func_register_mapping.iter() {
            if !inst_register_mapping.contains_key(key) {
                inst_register_mapping.insert(key.clone(), value.clone());
            }
        }
    }

    inst_register_mapping
}

fn add_dominance_path(irgm: &mut IRGraphManager, path: PathBuf) {
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
    let file_name = path.to_str().unwrap().trim_end_matches(".txt").to_owned()
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
}

pub fn run_file(file_name: String) {
    let mut path = PathBuf::new();
    path.push(env::current_exe().unwrap());
    path.pop();
    path.pop();
    path.pop();
    path.pop(); //this is needed because current .exe is 4 folders too deep.
    path.push("src/Testing/");
    let mut base_path = path.clone();
    path.push(file_name.clone() + ".txt");
    println!("{:?}", path);

    let file = fs::File::open(path.as_path()).expect("Error Opening File.");

    let mut buffer = String::new();
    let result = BufReader::new(file).read_to_string(&mut buffer);

    let mut char_iter = buffer.chars().peekable();
    let mut read_iter = char_iter.clone();

    let tokens = Lexer::token::TokenCollection::collect(&mut read_iter);
    let mut tc = Lexer::token::TokenCollection::collect(&mut char_iter);

    let comp = Parser::AST::computation::Comp::new(&mut tc);
    let irgmanager = comp.to_ir();

    let mut optimizer = Optimizer::Optimizer::new(irgmanager);
    optimizer.pass_0();
    optimizer.pass_1();
    optimizer.pass_2();
    optimizer.pass_3();

    //clean_graph(&mut irgmanager);
    //Optimizer::constant_evaluation::eval_program_constants(&mut irgmanager);

    let mut irgmanager = optimizer.get_irgm();

    // Getting back irgm from the optimizer.
    let root_node = irgmanager.graph_manager().get_main_node();
    let exit_nodes = irgmanager.graph_manager().get_exit_nodes(&root_node);

    for exit_id in exit_nodes {
        //analyze_live_range(&mut irgmanager, root_node.clone(), exit_id, path.clone(), file_name.clone());
    }

    /// TEST SPACE FOR Dominators
    ///
    /// It works!
    let root = irgmanager.graph_manager().get_main_node();
    let graph = irgmanager.graph_manager().get_mut_ref_graph().clone();
    let dom_space = simple_fast(&graph, root);

    //println!("{:?}", dom_space);
    for node in graph.node_indices() {
        match dom_space.immediate_dominator(node) {
            Some(parent_node) => {
                irgmanager
                    .graph_manager()
                    .add_dominance_edge(node, parent_node);
            }
            None => {}
        }
    }

    /// END TEST SPACE ///
    base_path.push(file_name + ".dot");

    let mut output = String::new();
    write!(
        output,
        "{:?}",
        display::Dot::with_config(
            &irgmanager.graph_manager().get_mut_ref_graph().clone(),
            &[display::Config::EdgeColor]
        )
    );
    fs::write(base_path.as_path(), output);

    println!();
    println!();
    println!();
}

pub fn clean_base_values(irgm: &mut IRGraphManager) {
    for node in irgm.graph_manager().get_mut_ref_graph().node_weights_mut() {
        for inst in node.get_mut_data_ref().get_mut_inst_list_ref() {
            inst.borrow_mut().update_base_values();
        }
    }
}

pub fn extract_constants(irgm: &mut IRGraphManager, root_id: NodeIndex) {
    // Go through each node:
    //  - If inst contains too many constants,
    //    remove them to immediately above
    //  - If it is a phi instruction, there are
    //    two cases:
    //      - if -> split up in whichever direction
    //              the replaced constant comes from
    //              as an add or subtract inst.
    //      - while -> the constant should only
    //              be in the immediate dominator
    //              path, but if there is a case of
    //              two constants.... well that
    //              will suck....
    //  - Once all constants are properly removed,
    //    continue with other functions. Should
    //    behave... hopefully.

    let dom_space = simple_fast(&irgm.graph_manager_ref().get_ref_graph(), root_id.clone());
    let visit_pattern = irgm.graph_manager().graph_visitor(root_id.clone());

    for node_id in visit_pattern.iter() {
        let mut x_inst_push_list = Vec::new();
        let mut y_inst_push_list = Vec::new();
        // for replacing x inst 0, y inst 1
        let mut std_inst_insert = Vec::new();

        for (inst_position, inst) in irgm
            .graph_manager()
            .get_mut_ref_graph()
            .node_weight_mut(node_id.clone())
            .unwrap()
            .get_mut_data_ref()
            .get_mut_inst_list_ref()
            .iter()
            .enumerate()
        {
            let inst_ty = inst.borrow().inst_type().clone();
            let inst_id = inst.borrow().get_inst_num();

            let inst_values = inst.borrow().get_val_ty();

            match inst_values {
                // x_val is const, y_val is const
                (Some(ValTy::con(x_val)), Some(ValTy::con(y_val))) => {
                    match inst_ty {
                        InstTy::add | InstTy::sub | InstTy::mul | InstTy::div | InstTy::cmp => {
                            if x_val == 0 {
                                inst.borrow_mut().update_x_val(Value::new(ValTy::reg(
                                    RegisterAllocation::allocate_R0(),
                                )));
                            } else {
                                std_inst_insert.push((x_val, inst_position, Rc::clone(inst), 0));
                            }

                            if y_val == 0 {
                                inst.borrow_mut().update_y_val(Value::new(ValTy::reg(
                                    RegisterAllocation::allocate_R0(),
                                )));
                            }
                        }
                        InstTy::phi => {
                            // The complicated instruction
                            if x_val == 0 {
                                inst.borrow_mut().update_x_val(Value::new(ValTy::reg(
                                    RegisterAllocation::allocate_R0(),
                                )));
                            } else {
                                x_inst_push_list.push((x_val, Rc::clone(inst)));
                            }

                            if y_val == 0 {
                                inst.borrow_mut().update_y_val(Value::new(ValTy::reg(
                                    RegisterAllocation::allocate_R0(),
                                )));
                            } else {
                                y_inst_push_list.push((y_val, Rc::clone(inst)));
                            }
                        }
                        _ => {
                            // All other instructions cant have any constants
                            if x_val == 0 {
                                inst.borrow_mut().update_x_val(Value::new(ValTy::reg(
                                    RegisterAllocation::allocate_R0(),
                                )));
                            } else {
                                std_inst_insert.push((x_val, inst_position, Rc::clone(inst), 0));
                            }

                            if y_val == 0 {
                                inst.borrow_mut().update_y_val(Value::new(ValTy::reg(
                                    RegisterAllocation::allocate_R0(),
                                )));
                            } else {
                                std_inst_insert.push((y_val, inst_position, Rc::clone(inst), 1));
                            }
                        }
                    }
                }
                // x_val is const, y_val we dont care about. Just handle x being some const
                (Some(ValTy::con(x_val)), _) => {
                    match inst_ty {
                        InstTy::phi => {
                            if x_val == 0 {
                                inst.borrow_mut().update_x_val(Value::new(ValTy::reg(
                                    RegisterAllocation::allocate_R0(),
                                )));
                            } else {
                                x_inst_push_list.push((x_val, Rc::clone(inst)));
                            }
                        }
                        InstTy::ret => {
                            if x_val == 0 {
                                inst.borrow_mut().update_x_val(Value::new(ValTy::reg(
                                    RegisterAllocation::allocate_R0(),
                                )));
                            } else {
                                std_inst_insert.push((x_val, inst_position, Rc::clone(inst), 0));
                            }
                        }
                        _ => {
                            // All other just x instructions should be handled the same way.
                            if x_val == 0 {
                                inst.borrow_mut().update_x_val(Value::new(ValTy::reg(
                                    RegisterAllocation::allocate_R0(),
                                )));
                            } else {
                                std_inst_insert.push((x_val, inst_position, Rc::clone(inst), 0));
                            }
                        }
                    }
                }
                // x_val we dont care about, y_val is const. Just handle y being some const
                (_, Some(ValTy::con(y_val))) => {
                    match inst_ty {
                        InstTy::add | InstTy::sub | InstTy::mul | InstTy::div | InstTy::cmp => {
                            // Pass through on this case, as this is acceptable behavior
                            if y_val == 0 {
                                inst.borrow_mut().update_y_val(Value::new(ValTy::reg(
                                    RegisterAllocation::allocate_R0(),
                                )));
                            }
                        }
                        InstTy::phi => {
                            if y_val == 0 {
                                inst.borrow_mut().update_y_val(Value::new(ValTy::reg(
                                    RegisterAllocation::allocate_R0(),
                                )));
                            } else {
                                y_inst_push_list.push((y_val, Rc::clone(inst)));
                            }
                        }
                        _ => {
                            if y_val == 0 {
                                inst.borrow_mut().update_y_val(Value::new(ValTy::reg(
                                    RegisterAllocation::allocate_R0(),
                                )));
                            } else {
                                std_inst_insert.push((y_val, inst_position, Rc::clone(inst), 1));
                            }
                        }
                    }
                }
                _ => {
                    // These are all instructions that do not have constants.
                }
            }
        }

        // First handle all instructions that need to be added within this block
        std_inst_insert.reverse();
        for (val, position, inst_clone, x_y) in std_inst_insert.iter() {
            let block_id = irgm
                .graph_manager()
                .get_ref_graph()
                .node_weight(node_id.clone())
                .unwrap()
                .get_node_id();

            let add_inst = irgm.build_op_x_y_in_block(
                Value::new(ValTy::reg(RegisterAllocation::allocate_R0())),
                Value::new(ValTy::con(val.clone())),
                InstTy::add,
                block_id,
            );

            let inst_val = irgm.graph_manager().insert_instruction_in_node(
                position.clone(),
                add_inst,
                node_id,
            );
            if x_y.clone() == 0 {
                // Replacing the x instruction
                inst_clone.borrow_mut().update_x_val(inst_val);
            } else {
                // Replacing the y instruction
                inst_clone.borrow_mut().update_y_val(inst_val);
            }

            //println!("Inserting Op: {:?} in position: {} for Inst: {:?}", add_inst, position, temp_manager.borrow_inst(inst_id));
        }

        if x_inst_push_list.is_empty() && y_inst_push_list.is_empty() {
            // Early bail out if there are no x or y phi values to adjust
            //println!("Returning Early!");
            continue;
        }

        let parents = irgm
            .graph_manager()
            .get_ref_graph()
            .neighbors_directed(node_id.clone(), Incoming)
            .map(|node_id| node_id)
            .collect::<Vec<NodeIndex>>();

        let mut is_while = false;
        let mut ordered_parents = Vec::new();

        // This gives both information as to which control flow type it
        // is, as well as sorting for the while case.
        for parent_id in parents.iter() {
            if dom_space.immediate_dominator(node_id.clone()) == Some(parent_id.clone()) {
                ordered_parents.insert(0, parent_id.clone());
                is_while = true;
            } else {
                ordered_parents.push(parent_id.clone());
            }
        }
        ordered_parents.reverse();
        // X is now always path 0, and Y is always path 1

        //println!("Parents of node: {:?}\n {:?}", node_id, ordered_parents);

        // Second, handle all the x instructions
        // These can be placed as they were found.
        for (val, inst_clone) in x_inst_push_list.iter() {
            let parent_node_id = ordered_parents[0].clone();

            let parent_block_id = irgm
                .graph_manager()
                .get_ref_graph()
                .node_weight(parent_node_id)
                .unwrap()
                .get_node_id();

            let add_inst = irgm.build_op_x_y_in_block(
                Value::new(ValTy::reg(RegisterAllocation::allocate_R0())),
                Value::new(ValTy::con(val.clone())),
                InstTy::add,
                parent_block_id,
            );
            let inst_val = irgm
                .graph_manager()
                .add_instruction_in_node(add_inst, &parent_node_id);

            inst_clone.borrow_mut().update_x_val(inst_val);
        }

        // Third, handle all the y instructions
        // These can also be placed as they are found
        for (val, inst_clone) in y_inst_push_list.iter() {
            let parent_node_id = ordered_parents[1].clone();

            let parent_block_id = irgm
                .graph_manager()
                .get_ref_graph()
                .node_weight(parent_node_id)
                .unwrap()
                .get_node_id();

            let add_inst = irgm.build_op_x_y_in_block(
                Value::new(ValTy::reg(RegisterAllocation::allocate_R0())),
                Value::new(ValTy::con(val.clone())),
                InstTy::add,
                parent_block_id,
            );

            let inst_val = irgm
                .graph_manager()
                .add_instruction_in_node(add_inst, &parent_node_id);

            inst_clone.borrow_mut().update_y_val(inst_val);
        }
    }
}
