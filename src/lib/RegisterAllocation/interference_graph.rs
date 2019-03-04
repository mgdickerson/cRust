use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;

use super::{Color, Node, NodeIndex};
use lib::Utility::display;
use lib::IR::ir::ValTy;
use lib::IR::ir::{InstTy, Op};
use lib::IR::ir_manager::IRGraphManager;

use petgraph::algo::dominators::simple_fast;
use petgraph::algo::dominators::Dominators;
use petgraph::Graph;
use petgraph::{Directed, Incoming, Outgoing};

use std::collections::HashMap;
use std::fmt::Debug;
use std::fmt::Write;
use std::fs::OpenOptions;
use std::env;
use std::fs::{self, DirEntry};
use std::io::prelude::*;
use std::io::{BufRead, BufReader, Result};
use std::path::Path;
use std::path::PathBuf;
use std::ffi::OsString;

pub struct InterferenceGraph {
    inter_graph: Graph<OpNode, String, Directed, u32>,
}

impl InterferenceGraph {}

pub struct OpNode {
    inst: Rc<RefCell<Op>>,
    reg_color: Option<Color>,
}

impl OpNode {
    pub fn new(inst: Rc<RefCell<Op>>) -> Self {
        OpNode {
            inst,
            reg_color: None,
        }
    }

    pub fn get_inst_ref(&self) -> &Rc<RefCell<Op>> {
        &self.inst
    }
}

impl Debug for OpNode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "({}): {}",
            self.inst.borrow().get_inst_num(),
            self.inst.borrow().to_string()
        )
    }
}

pub fn analyze_live_range(irgm: &mut IRGraphManager, root_node: NodeIndex, exit_node: NodeIndex, path: PathBuf, entry: OsString) {
    // Make vector of live instructions.
    // When a new instruction is found that is not
    // part of the "live" instructions, add it to
    // the list and add an edge to it to all other
    // live instructions.
    //let mut live_values = Vec::new();

    // Create a new graph which will contain each instruction as a node,
    // and edges between instructions represent the interference.

    let graph = irgm.graph_manager().get_mut_ref_graph().clone();
    let dom_space = simple_fast(&graph, root_node.clone());

    let mut recurse_graph = RecurseTraverse::new(
        exit_node,
        dom_space
    );

    recurse_graph.recursive_traversal(irgm);
    let interference_graph = recurse_graph.get_interference_graph();

    let mut dot_graph_path = entry;
    let mut file_name = path.to_str().unwrap().to_owned()
        + "/"
        + dot_graph_path.to_str().unwrap().trim_end_matches(".txt")
        + "_interference.dot";

    let mut output = String::new();
    write!(
        output,
        "{:?}",
        display::Dot::with_config(
            &interference_graph,
            &[display::Config::EdgeNoLabel]
        )
    );
    fs::write(file_name, output);

    // TODO : No longer need recurse base, just use the exit points
    // on the graphs provided. Those should be the correct exit points
    // for the graph and thus should work for bottom up traversal.
}

struct RecurseTraverse {
    current_node: NodeIndex,
    interference_graph: Graph<OpNode,String,Directed,u32>,
    inst_node_map: HashMap<usize, NodeIndex>,
    live_inst_map: HashMap<usize, NodeIndex>,
    dominators: Dominators<NodeIndex>,
    while_bp: Option<NodeIndex>,
    if_bp: Option<NodeIndex>,
}

impl RecurseTraverse {
    fn new(current_node: NodeIndex, dominators: Dominators<NodeIndex>) -> Self {
        RecurseTraverse {
            current_node,
            interference_graph: Graph::new(),
            inst_node_map: HashMap::new(),
            live_inst_map: HashMap::new(),
            dominators,
            while_bp: None,
            if_bp: None,
        }
    }

    pub fn get_interference_graph(self) -> Graph<OpNode,String,Directed,u32> {
        self.interference_graph
    }

    /// Recursing options:
    /// 0-Parents                    -> End of recursion, return completed live range
    /// 1-Parent                     -> Continue directly up graph
    /// 2-Parents (neither dominate) -> An if branch. go up left (until loop header is found), then traverse up right.
    ///                                 Continue with the loop header after joining live range from left and right.
    /// 2-Parents (one dominates)    -> An else branch, loop through the branch dominated by loop twice, then traverse
    ///                                 up the dominating path.
    fn recursive_traversal(&mut self, irgm: &mut IRGraphManager) {
        if let Some(node_id) = self.if_bp {
            if self.current_node == node_id.clone() {
                return
            }
        }

        let mut parents = Vec::new();
        for parent_id in irgm
            .clone()
            .graph_manager()
            .get_ref_graph()
            .neighbors_directed(self.current_node.clone(), Incoming) {
            parents.push(parent_id);
        }

        match parents.len() {
            0 => {
                // Final node, perform any required actions then simply return.
                self.grab_live_ranges(irgm, BlockType::standard);
                return;
            },
            1 => {
                self.grab_live_ranges(irgm, BlockType::standard);
                self.current_node = parents.pop().unwrap();
                self.recursive_traversal(irgm);
                return
            },
            2 => {
                // two cases here, if or while
                let mut ordered_parents = Vec::new();
                let mut is_while = false;

                // This gives both information as to which control flow type it
                // is, as well as sorting for the while case.
                for node_id in parents.iter() {
                    if self.dominators.immediate_dominator(self.current_node) == Some(node_id.clone()) {
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
                    if let Some(node_id) = &self.while_bp {
                        if self.current_node == node_id.clone() {
                            return
                        }
                    }

                    // Make save point of previous while break point
                    let prev_while_bp = self.while_bp.clone();
                    let current_id_recovery = self.current_node.clone();

                    // Set while break point
                    self.grab_live_ranges(irgm, BlockType::while_loop);
                    self.while_bp = Some(self.current_node.clone());
                    self.current_node = ordered_parents[1].clone();
                    self.recursive_traversal(irgm);

                    // Grab live ranges for the loop of the while
                    self.current_node = current_id_recovery;
                    self.grab_live_ranges(irgm, BlockType::while_loop);
                    self.current_node = ordered_parents[1].clone();
                    self.recursive_traversal(irgm);

                    // This is the final run through the current node, this time only grab the right side of any phis.
                    // Grab live ranges for the loop of the while
                    self.current_node = current_id_recovery;
                    self.grab_live_ranges(irgm, BlockType::while_cont);
                    self.while_bp = prev_while_bp;
                    self.current_node = ordered_parents[0].clone();
                    self.recursive_traversal(irgm);
                    return
                }
                else {
                    //println!("Is If Case!");
                    //println!("Ordered parents: {:?}", ordered_parents);

                    // This is the if case. Traverse up both paths until the dominator is hit, then return
                    // and merge the two live ranges and go through the dominating path.
                    let immediate_dominator = self.dominators
                        .immediate_dominator(self.current_node)
                        .expect(&format!("No dominating path found for: {:?}", self.current_node)[..])
                        .clone();

                    // Clone current live range so it can be used for both left and right side
                    let live_range_copy = self.live_inst_map.clone();
                    let if_bp_recovery = self.if_bp.clone();
                    let current_node_recovery = self.current_node.clone();

                    // Grab live range of this block for right case
                    self.grab_live_ranges(irgm, BlockType::if_phi_right);
                    self.if_bp = Some(immediate_dominator.clone());
                    self.current_node = ordered_parents[0].clone();

                    self.recursive_traversal(irgm);

                    // Save new live range, recover old one
                    let mut final_live_range = self.live_inst_map.clone();
                    self.live_inst_map = live_range_copy;
                    self.current_node = current_node_recovery;

                    self.grab_live_ranges(irgm, BlockType::if_phi_left);
                    self.current_node = ordered_parents[1].clone();

                    self.recursive_traversal(irgm);

                    /*println!("Live Range of Else Branch");
                    let mut live_range_str = String::new();
                    for (_, live_inst) in live_path_0_map.iter() {
                        if live_range_str.is_empty() {
                            live_range_str += &String::from(format!("[ {:?}", interference_graph.node_weight(live_inst.clone()).unwrap().get_inst_ref().borrow().get_inst_num()));
                        } else {
                            live_range_str += &String::from(format!(", {:?}", interference_graph.node_weight(live_inst.clone()).unwrap().get_inst_ref().borrow().get_inst_num()));
                        }
                    }
                    if !live_range_str.is_empty() {
                        live_range_str += " ]";
                    }

                    println!("{}", live_range_str);

                    println!("Live Range of If Branch");
                    let mut live_range_str = String::new();
                    for (_, live_inst) in live_path_1_map.iter() {
                        if live_range_str.is_empty() {
                            live_range_str += &String::from(format!("[ {:?}", interference_graph.node_weight(live_inst.clone()).unwrap().get_inst_ref().borrow().get_inst_num()));
                        } else {
                            live_range_str += &String::from(format!(", {:?}", interference_graph.node_weight(live_inst.clone()).unwrap().get_inst_ref().borrow().get_inst_num()));
                        }
                    }
                    if !live_range_str.is_empty() {
                        live_range_str += " ]";
                    }

                    println!("{}", live_range_str);*/

                    // The current live_inst_map should contain all the live range information from the if branch
                    // and so simply comparing with the stored final_live_range should get a completed list.
                    for (key, value) in self.live_inst_map.iter() {
                        if !final_live_range.contains_key(key) {
                            final_live_range.insert(key.clone(), value.clone());
                        }
                    }

                    // Once the two lists have been combined, make it the live_inst_map
                    self.live_inst_map = final_live_range;

                    // set next node to the immediate_dominator node
                    self.if_bp = if_bp_recovery;
                    self.current_node = immediate_dominator;

                    /*println!("Combined Live Range");
                    let mut live_range_str = String::new();
                    for (_, live_inst) in live_path_1_map.iter() {
                        if live_range_str.is_empty() {
                            live_range_str += &String::from(format!("[ {:?}", interference_graph.node_weight(live_inst.clone()).unwrap().get_inst_ref().borrow().get_inst_num()));
                        } else {
                            live_range_str += &String::from(format!(", {:?}", interference_graph.node_weight(live_inst.clone()).unwrap().get_inst_ref().borrow().get_inst_num()));
                        }
                    }
                    if !live_range_str.is_empty() {
                        live_range_str += " ]";
                    }

                    println!("{}", live_range_str);*/

                    self.recursive_traversal(irgm);

                    return
                }
            },
            _ => {
                panic!("Should be no more than 2 parents for any given node of the graph.");
            },
        }
    }

    fn grab_live_ranges(&mut self, irgm: &mut IRGraphManager, block_type: BlockType) {

        // Get current node's instructions
        let mut inst_list = irgm
            .graph_manager()
            .get_ref_graph()
            .node_weight(self.current_node.clone())
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
            if self.live_inst_map.contains_key(&inst_id) {
                self.live_inst_map.remove(&inst_id);
            }

            // Check for x and y values, only Ops can produce result and must be tracked.

            // Check for an x_value
            if let Some(x_val) = inst.borrow().clone_x_val() {
                if let ValTy::op(x_inst) = x_val.get_value() {
                    let x_inst_id = x_inst.borrow().get_inst_num();
                    if !self.live_inst_map.contains_key(&x_inst_id) {
                        // This instruction is not already part of the live range.
                        // Create new node and add to the graph.
                        let op_node = OpNode::new(Rc::clone(x_inst));

                        let inst_node_id;
                        if !self.inst_node_map.contains_key(&x_inst_id) {
                            inst_node_id = self.interference_graph.add_node(op_node);
                            self.inst_node_map.insert(x_inst_id, inst_node_id.clone());
                        } else {
                            inst_node_id = self.inst_node_map.get(&x_inst_id).unwrap().clone();
                        }

                        if inst_type == InstTy::phi {
                            if block_type == BlockType::while_loop
                                || block_type == BlockType::if_phi_left
                            {
                                // Make an edge between all nodes currently in live range, then add to live range
                                for (_, node_id) in self.live_inst_map.iter() {
                                    if None
                                        == self.interference_graph
                                        .find_edge_undirected(inst_node_id, node_id.clone())
                                    {
                                        self.interference_graph.update_edge(
                                            inst_node_id,
                                            node_id.clone(),
                                            String::from("black"),
                                        );
                                    }
                                }

                                self.live_inst_map.insert(x_inst_id, inst_node_id);
                            } else {
                                // Do not add to the liveness range, fall through to y.
                            }
                        } else {
                            // Make an edge between all nodes currently in live range, then add to live range
                            for (_, node_id) in self.live_inst_map.iter() {
                                if None
                                    == self.interference_graph
                                    .find_edge_undirected(inst_node_id, node_id.clone())
                                {
                                    self.interference_graph.update_edge(
                                        inst_node_id,
                                        node_id.clone(),
                                        String::from("black"),
                                    );
                                }
                            }

                            self.live_inst_map.insert(x_inst_id, inst_node_id);
                        }
                    }
                }
            }

            // Check for a y_value
            if let Some(y_val) = inst.borrow().clone_y_val() {
                if let ValTy::op(y_inst) = y_val.get_value() {
                    let y_inst_id = y_inst.borrow().get_inst_num();
                    if !self.live_inst_map.contains_key(&y_inst_id) {
                        // This instruction is not already part of the live range.
                        // Create new node and add to the graph.
                        let op_node = OpNode::new(Rc::clone(y_inst));

                        let inst_node_id;
                        if !self.inst_node_map.contains_key(&y_inst_id) {
                            inst_node_id = self.interference_graph.add_node(op_node);
                            self.inst_node_map.insert(y_inst_id, inst_node_id.clone());
                        } else {
                            inst_node_id = self.inst_node_map.get(&y_inst_id).unwrap().clone();
                        }

                        if inst_type == InstTy::phi {
                            if block_type == BlockType::if_phi_right
                                || block_type == BlockType::while_cont
                            {
                                // Make an edge between all nodes currently in live range, then add to live range
                                for (_, node_id) in self.live_inst_map.iter() {
                                    if None
                                        == self.interference_graph
                                        .find_edge_undirected(inst_node_id, node_id.clone())
                                    {
                                        self.interference_graph.update_edge(
                                            inst_node_id,
                                            node_id.clone(),
                                            String::from("black"),
                                        );
                                    }
                                }

                                self.live_inst_map.insert(y_inst_id, inst_node_id);
                            } else {
                                // Dont add anything other than if_phi_right. Then continue.
                            }
                        } else {
                            // Make an edge between all nodes currently in live range, then add to live range
                            for (_, node_id) in self.live_inst_map.iter() {
                                if None
                                    == self.interference_graph
                                    .find_edge_undirected(inst_node_id, node_id.clone())
                                {
                                    self.interference_graph.update_edge(
                                        inst_node_id,
                                        node_id.clone(),
                                        String::from("black"),
                                    );
                                }
                            }

                            self.live_inst_map.insert(y_inst_id, inst_node_id);
                        }
                    }
                }
            }
        }

        println!("Live range ending at node {} [Type: {:?}]:", irgm.graph_manager().get_ref_graph().node_weight(self.current_node).unwrap().get_node_id(), block_type);
        let mut live_range_str = String::new();
        for (_, live_inst) in self.live_inst_map.iter() {
            if live_range_str.is_empty() {
                live_range_str += &String::from(format!("[ {:?}", self.interference_graph.node_weight(live_inst.clone()).unwrap().get_inst_ref().borrow().get_inst_num()));
            } else {
                live_range_str += &String::from(format!(", {:?}", self.interference_graph.node_weight(live_inst.clone()).unwrap().get_inst_ref().borrow().get_inst_num()));
            }
        }
        if !live_range_str.is_empty() {
            live_range_str += " ]";
        }

        println!("{}", live_range_str);

    }
}


#[derive(PartialEq,Debug)]
enum BlockType {
    standard,
    while_loop,
    while_cont,
    if_phi_left,
    if_phi_right,
}

/*

fn recurse_graph(
    irgm: &mut IRGraphManager,
    current_node: NodeIndex,
    interference_graph: &mut Graph<OpNode, String, Directed, u32>,
    inst_node_map: &mut HashMap<usize, NodeIndex>,
    live_node_map: &mut HashMap<usize, NodeIndex>,
    dominators: &Dominators<NodeIndex>,
    while_break_point: Option<NodeIndex>,
    if_break_point: Option<NodeIndex>
) -> HashMap<usize, NodeIndex> {
    // Ensure that the if loops dont pass the dominating node
    if let Some(node_id) = if_break_point.clone() {
        if current_node == node_id {
            return live_node_map.clone();
        }
    }

    // Get parents from current node.
    let mut parents = Vec::new();
    *//*println!(
        "Parents of node: {:?}",
        irgm.graph_manager()
            .get_ref_graph()
            .node_weight(current_node.clone())
            .unwrap()
            .get_node_id()
    );*//*
    for parent_id in irgm
        .clone()
        .graph_manager()
        .get_ref_graph()
        .neighbors_directed(current_node.clone(), Incoming)
    {
        parents.push(parent_id);
        *//*println!(
            "Parent node: {:?}",
            irgm.graph_manager()
                .get_ref_graph()
                .node_weight(parent_id.clone())
                .unwrap()
                .get_node_id()
        );*//*
    }

    match parents.len() {
        0 => {
            // Grab live ranges from current node
            grab_live_ranges(
                irgm,
                current_node,
                interference_graph,
                inst_node_map,
                live_node_map,
                BlockType::standard,
            );

            // Final node, perform any required actions then simply return.
            return live_node_map.clone();
        }
        1 => {
            // Grab live ranges from current node
            grab_live_ranges(
                irgm,
                current_node,
                interference_graph,
                inst_node_map,
                live_node_map,
                BlockType::standard,
            );

            let current_node = parents.pop().unwrap();
            recurse_graph(
                irgm,
                current_node,
                interference_graph,
                inst_node_map,
                live_node_map,
                dominators,
                while_break_point,
                if_break_point,
            );
            return live_node_map.clone();
        }
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
                if let Some(node_id) = while_break_point.clone() {
                    *//*println!(
                        "Loop Break Point: {:?} -> Current Node: {:?}",
                        node_id, current_node
                    );*//*
                    if current_node == node_id {
                        return live_node_map.clone();
                    }
                }

                // Grab live ranges for the loop of the while
                let previous_break_point = while_break_point.clone();
                grab_live_ranges(
                    irgm,
                    current_node,
                    interference_graph,
                    inst_node_map,
                    live_node_map,
                    BlockType::while_loop,
                );
                let mut live_node_map_1 = recurse_graph(
                    irgm,
                    ordered_parents[1].clone(),
                    interference_graph,
                    inst_node_map,
                    live_node_map,
                    dominators,
                    Some(current_node.clone()),
                    if_break_point,
                );

                // Grab live ranges for the loop of the while
                grab_live_ranges(
                    irgm,
                    current_node,
                    interference_graph,
                    inst_node_map,
                    &mut live_node_map_1,
                    BlockType::while_loop,
                );
                let mut live_node_map_2 = recurse_graph(
                    irgm,
                    ordered_parents[1].clone(),
                    interference_graph,
                    inst_node_map,
                    &mut live_node_map_1,
                    dominators,
                    Some(current_node.clone()),
                    if_break_point,
                );

                // This is the final run through the current node, this time only grab the right side of any phis.
                // Grab live ranges for the loop of the while
                grab_live_ranges(
                    irgm,
                    current_node,
                    interference_graph,
                    inst_node_map,
                    &mut live_node_map_2,
                    BlockType::while_cont,
                );
                recurse_graph(
                    irgm,
                    ordered_parents[0].clone(),
                    interference_graph,
                    inst_node_map,
                    &mut live_node_map_2,
                    dominators,
                    previous_break_point,
                    if_break_point,
                );
                return live_node_map.clone();
            } else {
                //println!("Is If Case!");
                //println!("Ordered parents: {:?}", ordered_parents);

                // This is the if case. Traverse up both paths until the dominator is hit, then return
                // and merge the two live ranges and go through the dominating path.
                let immediate_dominator = dominators
                    .immediate_dominator(current_node)
                    .expect(&format!("No dominating path found for: {:?}", current_node)[..])
                    .clone();
                let current_if_break_point = if_break_point.clone();

                let mut live_path_0_map = live_node_map.clone();
                let mut live_path_1_map = live_node_map.clone();

                // Grab live ranges for the left side of phi
                grab_live_ranges(
                    irgm,
                    current_node,
                    interference_graph,
                    inst_node_map,
                    &mut live_path_0_map,
                    BlockType::if_phi_right,
                );
                live_path_0_map = recurse_graph(
                    irgm,
                    ordered_parents[0].clone(),
                    interference_graph,
                    inst_node_map,
                    &mut live_path_0_map,
                    dominators,
                    while_break_point,
                    Some(immediate_dominator.clone()),
                );



                // Grab live ranges for the left side of phi
                grab_live_ranges(
                    irgm,
                    current_node,
                    interference_graph,
                    inst_node_map,
                    &mut live_path_1_map,
                    BlockType::if_phi_left,
                );
                live_path_1_map = recurse_graph(
                    irgm,
                    ordered_parents[1].clone(),
                    interference_graph,
                    inst_node_map,
                    &mut live_path_1_map,
                    dominators,
                    while_break_point,
                    Some(immediate_dominator.clone()),
                );

                println!("Live Range of Else Branch");
                let mut live_range_str = String::new();
                for (_, live_inst) in live_path_0_map.iter() {
                    if live_range_str.is_empty() {
                        live_range_str += &String::from(format!("[ {:?}", interference_graph.node_weight(live_inst.clone()).unwrap().get_inst_ref().borrow().get_inst_num()));
                    } else {
                        live_range_str += &String::from(format!(", {:?}", interference_graph.node_weight(live_inst.clone()).unwrap().get_inst_ref().borrow().get_inst_num()));
                    }
                }
                if !live_range_str.is_empty() {
                    live_range_str += " ]";
                }

                println!("{}", live_range_str);

                println!("Live Range of If Branch");
                let mut live_range_str = String::new();
                for (_, live_inst) in live_path_1_map.iter() {
                    if live_range_str.is_empty() {
                        live_range_str += &String::from(format!("[ {:?}", interference_graph.node_weight(live_inst.clone()).unwrap().get_inst_ref().borrow().get_inst_num()));
                    } else {
                        live_range_str += &String::from(format!(", {:?}", interference_graph.node_weight(live_inst.clone()).unwrap().get_inst_ref().borrow().get_inst_num()));
                    }
                }
                if !live_range_str.is_empty() {
                    live_range_str += " ]";
                }

                println!("{}", live_range_str);

                // Combine live ranges of both paths here. The new live_path_1_map should be the new liveness range.
                for (key, value) in live_path_0_map.iter() {
                    if !live_path_1_map.contains_key(key) {
                        live_path_1_map.insert(key.clone(), value.clone());
                    }
                }

                println!("Combined Live Range");
                let mut live_range_str = String::new();
                for (_, live_inst) in live_path_1_map.iter() {
                    if live_range_str.is_empty() {
                        live_range_str += &String::from(format!("[ {:?}", interference_graph.node_weight(live_inst.clone()).unwrap().get_inst_ref().borrow().get_inst_num()));
                    } else {
                        live_range_str += &String::from(format!(", {:?}", interference_graph.node_weight(live_inst.clone()).unwrap().get_inst_ref().borrow().get_inst_num()));
                    }
                }
                if !live_range_str.is_empty() {
                    live_range_str += " ]";
                }

                println!("{}", live_range_str);

                recurse_graph(
                    irgm,
                    immediate_dominator,
                    interference_graph,
                    inst_node_map,
                    &mut live_path_1_map,
                    dominators,
                    while_break_point,
                    current_if_break_point,
                );

                //println!("Reaches end of if.");
                return live_node_map.clone();
            }
        }
        _ => {
            panic!("Should be no more than 2 parents for any given node of the graph.");
        }
    }
}

fn grab_live_ranges(
    irgm: &mut IRGraphManager,
    current_node: NodeIndex,
    interference_graph: &mut Graph<OpNode, String, Directed, u32>,
    inst_node_map: &mut HashMap<usize, NodeIndex>,
    live_node_map: &mut HashMap<usize, NodeIndex>,
    block_type: BlockType,
) {
    println!("Live range starting at node {}:", irgm.graph_manager().get_ref_graph().node_weight(current_node).unwrap().get_node_id());
    let mut live_range_str = String::new();
    for (_, live_inst) in live_node_map.iter() {
        if live_range_str.is_empty() {
            live_range_str += &String::from(format!("[ {:?}", interference_graph.node_weight(live_inst.clone()).unwrap().get_inst_ref().borrow().get_inst_num()));
        } else {
            live_range_str += &String::from(format!(", {:?}", interference_graph.node_weight(live_inst.clone()).unwrap().get_inst_ref().borrow().get_inst_num()));
        }
    }
    if !live_range_str.is_empty() {
        live_range_str += " ]";
    }

    println!("{}", live_range_str);

    // Get current node's instructions
    let mut inst_list = irgm
        .graph_manager()
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
                        if block_type == BlockType::while_loop
                            || block_type == BlockType::if_phi_left
                        {
                            // Make an edge between all nodes currently in live range, then add to live range
                            for (_, node_id) in live_node_map.iter() {
                                if None
                                    == interference_graph
                                        .find_edge_undirected(inst_node_id, node_id.clone())
                                {
                                    interference_graph.update_edge(
                                        inst_node_id,
                                        node_id.clone(),
                                        String::from("black"),
                                    );
                                }
                            }

                            live_node_map.insert(x_inst_id, inst_node_id);
                        } else {
                            // Do not add to the liveness range, fall through to y.
                        }
                    } else {
                        // Make an edge between all nodes currently in live range, then add to live range
                        for (_, node_id) in live_node_map.iter() {
                            if None
                                == interference_graph
                                    .find_edge_undirected(inst_node_id, node_id.clone())
                            {
                                interference_graph.update_edge(
                                    inst_node_id,
                                    node_id.clone(),
                                    String::from("black"),
                                );
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
                        if block_type == BlockType::if_phi_right
                            || block_type == BlockType::while_cont
                        {
                            // Make an edge between all nodes currently in live range, then add to live range
                            for (_, node_id) in live_node_map.iter() {
                                if None
                                    == interference_graph
                                        .find_edge_undirected(inst_node_id, node_id.clone())
                                {
                                    interference_graph.update_edge(
                                        inst_node_id,
                                        node_id.clone(),
                                        String::from("black"),
                                    );
                                }
                            }

                            live_node_map.insert(y_inst_id, inst_node_id);
                        } else {
                            // Dont add anything other than if_phi_right. Then continue.
                        }
                    } else {
                        // Make an edge between all nodes currently in live range, then add to live range
                        for (_, node_id) in live_node_map.iter() {
                            if None
                                == interference_graph
                                    .find_edge_undirected(inst_node_id, node_id.clone())
                            {
                                interference_graph.update_edge(
                                    inst_node_id,
                                    node_id.clone(),
                                    String::from("black"),
                                );
                            }
                        }

                        live_node_map.insert(y_inst_id, inst_node_id);
                    }
                }
            }
        }
    }
}
*/

