use std::fmt;
use std::rc::Rc;
use std::cell::RefCell;

use super::{Node,NodeIndex,Color};
use lib::IR::ir::{Op, InstTy};
use lib::IR::ir_manager::IRGraphManager;
use lib::IR::ir::ValTy;
use lib::Utility::display;

use petgraph::Graph;
use petgraph::{Outgoing,Incoming, Directed};
use petgraph::algo::dominators::Dominators;
use petgraph::algo::dominators::simple_fast;
use std::collections::HashMap;
use std::fmt::Debug;

pub struct InterferenceGraph {
    inter_graph: Graph<OpNode,String,Directed,u32>,
}

impl InterferenceGraph {

}

pub struct OpNode {
    inst: Rc<RefCell<Op>>,
    reg_color: Option<Color>,
}

impl OpNode {
    pub fn new(inst: Rc<RefCell<Op>>) -> Self {
        OpNode { inst, reg_color: None }
    }
}

impl Debug for OpNode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({}): {}", self.inst.borrow().get_inst_num(), self.inst.borrow().to_string())
    }
}

pub fn analyze_live_range(
    irgm: &mut IRGraphManager,
    root_node: NodeIndex,
    exit_node: NodeIndex
) {
    // Make vector of live instructions.
    // When a new instruction is found that is not
    // part of the "live" instructions, add it to
    // the list and add an edge to it to all other
    // live instructions.
    //let mut live_values = Vec::new();

    // Create a new graph which will contain each instruction as a node,
    // and edges between instructions represent the interference.
    let mut interference_graph = Graph::new();
    let mut inst_node_map = HashMap::new();
    let mut live_node_map = HashMap::new();

    let graph = irgm.graph_manager().get_mut_ref_graph().clone();
    let dom_space = simple_fast(&graph,root_node.clone());

    recurse_graph(
        irgm,
        exit_node,
        &mut interference_graph,
        &mut inst_node_map,
        &mut live_node_map,
        & dom_space,
        None,
        false
    );

    println!("{:?}", display::Dot::with_config(&interference_graph, &[display::Config::EdgeNoLabel]));

    // TODO : No longer need recurse base, just use the exit points
    // on the graphs provided. Those should be the correct exit points
    // for the graph and thus should work for bottom up traversal.

}

/// Recursing options:
/// 0-Parents                    -> End of recursion, return completed live range
/// 1-Parent                     -> Continue directly up graph
/// 2-Parents (neither dominate) -> An if branch. go up left (until loop header is found), then traverse up right.
///                                 Continue with the loop header after joining live range from left and right.
/// 2-Parents (one dominates)    -> An else branch, loop through the branch dominated by loop twice, then traverse
///                                 up the dominating path.
fn recurse_graph(irgm: &mut IRGraphManager,
                 current_node: NodeIndex,
                 interference_graph: &mut Graph<OpNode,String,Directed,u32>,
                 inst_node_map: &mut HashMap<usize,NodeIndex>,
                 live_node_map: &mut HashMap<usize,NodeIndex>,
                 dominators: & Dominators<NodeIndex>,
                 loop_break_point: Option<NodeIndex>,
                 is_if: bool
) {
    // Ensure that the if loops dont pass the dominating node
    if is_if {
        if let Some(node_id) = loop_break_point.clone() {
            if current_node == current_node {
                return
            }
        }
    }

    // Get parents from current node.
    let mut parents = Vec::new();
    println!("Parents of node: {:?}", irgm.graph_manager().get_ref_graph().node_weight(current_node.clone()).unwrap().get_node_id());
    for parent_id in irgm.clone().graph_manager().get_ref_graph().neighbors_directed(current_node.clone(), Incoming) {
        parents.push(parent_id);
        println!("Parent node: {:?}", irgm.graph_manager().get_ref_graph().node_weight(parent_id.clone()).unwrap().get_node_id());
    }

    match parents.len() {
        0 => {
            // Grab live ranges from current node
            grab_live_ranges(irgm, current_node, interference_graph, inst_node_map, live_node_map, BlockType::standard);

            // Final node, perform any required actions then simply return.
            return
        },
        1 => {
            // Grab live ranges from current node
            grab_live_ranges(irgm, current_node, interference_graph, inst_node_map, live_node_map, BlockType::standard);

            let current_node = parents.pop().unwrap();
            recurse_graph(irgm, current_node, interference_graph, inst_node_map, live_node_map, dominators, loop_break_point, is_if);
            return
        },
        2 => {
            // two cases here, if or while
            let mut ordered_parents = Vec::new();
            let mut is_while = false;

            // This gives both information as to which control flow type it
            // is, as well as sorting for the while case.
            for node_id in parents.iter() {
                if dominators.immediate_dominator(current_node) == Some(node_id.clone()) {
                    ordered_parents.insert(0, node_id.clone());
                    is_while = true;
                } else {
                    ordered_parents.push(node_id.clone());
                }
            }

            if is_while {
                // This path will require two runs through the non-dominating path.
                // 0 is the dominating path, thus goes second.
                // 1 is the looping path, thus must go through it twice.

                // Ensure that the while loops dont recurse through the graph infinitely
                // but also goes through the header again.
                if let Some(node_id) = loop_break_point.clone() {
                    println!("Loop Break Point: {:?} -> Current Node: {:?}", node_id, current_node);
                    if current_node == node_id {
                        return
                    }
                }

                // Grab live ranges for the loop of the while
                let previous_break_point = loop_break_point.clone();
                grab_live_ranges(irgm, current_node, interference_graph, inst_node_map, live_node_map, BlockType::while_loop);
                recurse_graph(irgm,
                              ordered_parents[1].clone(),
                              interference_graph,
                              inst_node_map,
                              live_node_map,
                              dominators,
                              Some(current_node.clone()),
                              is_if);

                // Grab live ranges for the loop of the while
                grab_live_ranges(irgm, current_node, interference_graph, inst_node_map, live_node_map, BlockType::while_loop);
                recurse_graph(irgm,
                              ordered_parents[1].clone(),
                              interference_graph,
                              inst_node_map,
                              live_node_map,
                              dominators,
                              Some(current_node.clone()),
                              is_if);

                // This is the final run through the current node, this time only grab the right side of any phis.
                // Grab live ranges for the loop of the while
                grab_live_ranges(irgm, current_node, interference_graph, inst_node_map, live_node_map, BlockType::while_cont);
                recurse_graph(irgm, ordered_parents[0].clone(), interference_graph, inst_node_map, live_node_map, dominators, previous_break_point, is_if);
                return
            } else {
                // This is the if case. Traverse up both paths until the dominator is hit, then return
                // and merge the two live ranges and go through the dominating path.
                let immediate_dominator = dominators.immediate_dominator(current_node).expect(&format!("No dominating path found for: {:?}", current_node)[..]).clone();
                let is_if = true;


                let mut live_path_0_map = live_node_map.clone();

                // Grab live ranges for the left side of phi
                grab_live_ranges(irgm, current_node, interference_graph, inst_node_map, &mut live_path_0_map, BlockType::if_phi_left);
                recurse_graph(irgm,
                              ordered_parents[0].clone(),
                              interference_graph,
                              inst_node_map,
                              &mut live_path_0_map,
                              dominators,
                              Some(immediate_dominator.clone()),
                              is_if);


                let mut live_path_1_map = live_node_map.clone();

                // Grab live ranges for the left side of phi
                grab_live_ranges(irgm, current_node, interference_graph, inst_node_map, &mut live_path_1_map, BlockType::if_phi_right);
                recurse_graph(irgm, ordered_parents[1].clone(), interference_graph, inst_node_map, &mut live_path_1_map, dominators, Some(immediate_dominator.clone()), is_if);

                let is_if = false;

                // Combine live ranges of both paths here. The new live_path_1_map should be the new liveness range.
                for (key, value) in live_path_0_map.iter() {
                    if !live_path_1_map.contains_key(key) {
                        live_path_1_map.insert(key.clone(),value.clone());
                    }
                }

                recurse_graph(irgm, immediate_dominator, interference_graph, inst_node_map, &mut live_path_1_map, dominators, loop_break_point, is_if);
                return
            }
        },
        _ => {
            panic!("Should be no more than 2 parents for any given node of the graph.");
        }
    }
}

fn grab_live_ranges(
    irgm: &mut IRGraphManager,
    current_node: NodeIndex,
    interference_graph: &mut Graph<OpNode,String,Directed,u32>,
    inst_node_map: &mut HashMap<usize,NodeIndex>,
    live_node_map: &mut HashMap<usize,NodeIndex>,
    block_type: BlockType
) {
    // Get current node's instructions
    let mut inst_list = irgm.graph_manager()
        .get_ref_graph()
        .node_weight(current_node.clone())
        .unwrap()
        .get_data_ref()
        .get_inst_list_ref()
        .clone();

    // Reverse instruction to traverse inst from bottom to top
    inst_list.reverse();

    for inst in inst_list.iter() {
        // Get current instruction ID and remove from live range
        let inst_id = inst.borrow().get_inst_num();
        let inst_type = inst.borrow().inst_type().clone();

        // Remove instruction from live range
        if live_node_map.contains_key(&inst_id) {
            live_node_map.remove(&inst_id);
        }

        // Check for x and y values, only Ops can produce result and must be tracked.


        // Check for an x_value
        if let Some(x_val) = inst.borrow().clone_x_val() {
            if let ValTy::op(x_inst) = x_val.get_value() {
                let x_inst_id = x_inst.borrow().get_inst_num();
                if !live_node_map.contains_key(&x_inst_id) {
                    // This instruction is not already part of the live range.
                    // Create new node and add to the graph.
                    let op_node = OpNode::new(Rc::clone(x_inst));

                    let inst_node_id;
                    if !inst_node_map.contains_key(&x_inst_id) {
                        inst_node_id = interference_graph.add_node(op_node);
                        inst_node_map.insert(x_inst_id, inst_node_id.clone());
                    } else {
                        inst_node_id = inst_node_map.get(&x_inst_id).unwrap().clone();
                    }

                    if inst_type == InstTy::phi {
                        if block_type == BlockType::while_loop || block_type == BlockType::if_phi_left {
                            // Make an edge between all nodes currently in live range, then add to live range
                            for (_, node_id) in live_node_map.iter() {
                                if None == interference_graph.find_edge_undirected(inst_node_id, node_id.clone()) {
                                    interference_graph.update_edge(inst_node_id, node_id.clone(),
                                                                   String::from("black"));
                                }
                            }

                            live_node_map.insert(x_inst_id, inst_node_id);
                        } else {
                            // Do not add to the liveness range, fall through to y.
                        }
                    } else {
                        // Make an edge between all nodes currently in live range, then add to live range
                        for (_, node_id) in live_node_map.iter() {
                            if None == interference_graph.find_edge_undirected(inst_node_id, node_id.clone()) {
                                interference_graph.update_edge(inst_node_id, node_id.clone(),
                                                               String::from("black"));
                            }
                        }

                        live_node_map.insert(x_inst_id, inst_node_id);
                    }
                }
            }
        }

        // Check for a y_value
        if let Some(y_val) = inst.borrow().clone_y_val() {
            if let ValTy::op(y_inst) = y_val.get_value() {
                let y_inst_id = y_inst.borrow().get_inst_num();
                if !live_node_map.contains_key(&y_inst_id) {
                    // This instruction is not already part of the live range.
                    // Create new node and add to the graph.
                    let op_node = OpNode::new(Rc::clone(y_inst));

                    let inst_node_id;
                    if !inst_node_map.contains_key(&y_inst_id) {
                        inst_node_id = interference_graph.add_node(op_node);
                        inst_node_map.insert(y_inst_id, inst_node_id.clone());
                    } else {
                        inst_node_id = inst_node_map.get(&y_inst_id).unwrap().clone();
                    }

                    if inst_type == InstTy::phi {
                        if block_type == BlockType::if_phi_right || block_type == BlockType::while_cont {
                            // Make an edge between all nodes currently in live range, then add to live range
                            for (_, node_id) in live_node_map.iter() {
                                if None == interference_graph.find_edge_undirected(inst_node_id, node_id.clone()) {
                                    interference_graph.update_edge(inst_node_id, node_id.clone(), String::from("black"));
                                }
                            }

                            live_node_map.insert(y_inst_id, inst_node_id);
                        } else {
                            // Dont add anything other than if_phi_right. Then continue.
                        }
                    } else {
                        // Make an edge between all nodes currently in live range, then add to live range
                        for (_, node_id) in live_node_map.iter() {
                            if None == interference_graph.find_edge_undirected(inst_node_id, node_id.clone()) {
                                interference_graph.update_edge(inst_node_id, node_id.clone(), String::from("black"));
                            }
                        }

                        live_node_map.insert(y_inst_id, inst_node_id);
                    }
                }
            }
        }
    }
}

#[derive(PartialEq)]
enum BlockType {
    standard,
    while_loop,
    while_cont,
    if_phi_left,
    if_phi_right,
}