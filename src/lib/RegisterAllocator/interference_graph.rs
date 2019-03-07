use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;

use super::{RegisterAllocation, Color, Node, NodeIndex};
use lib::IR::ir::ValTy;
use lib::IR::ir::{InstTy, Op};
use lib::IR::ir_manager::IRGraphManager;

use petgraph::algo::dominators::Dominators;
use petgraph::Graph;
use petgraph::{Directed, Incoming, Outgoing};

use std::collections::HashMap;
use std::fmt::Debug;
use lib::Optimizer::temp_value_manager::TempValManager;

#[derive(Clone)]
pub struct OpNode {
    inst: Vec<Rc<RefCell<Op>>>,
    weight: usize,
    reg_color: Option<Color>,
    register: RegisterAllocation,
}

impl OpNode {
    pub fn new(inst: Rc<RefCell<Op>>, weight: usize) -> Self {
        let mut inst_vec = Vec::new();
        inst_vec.push(inst);
        OpNode {
            inst: inst_vec,
            weight,
            reg_color: None,

            // Give all nodes a temporary assignment to R0,
            // so if any node is missed it will give an error
            register: RegisterAllocation::allocate_R0()
        }
    }

    pub fn get_inst_ref(&self) -> &Vec<Rc<RefCell<Op>>> {
        &self.inst
    }

    pub fn coalesce_inst(&mut self, coal_op: Rc<RefCell<Op>>) {
        self.inst.push(coal_op);
    }

    pub fn add_weight(&mut self, weight: usize) {
        self.weight += weight;
    }

    pub fn get_weight(&self) -> usize {
        self.weight.clone()
    }

    fn add_color(&mut self, color: Color) {
        self.reg_color = Some(color);
    }

    pub fn assign_register(&mut self, reg: RegisterAllocation) {
        self.register = reg;
    }

    pub fn get_register(&self) -> usize {
        self.register.to_usize()
    }

    pub fn get_color(&mut self) {
        let local_reg_copy = self.register.clone();
        self.add_color(Color::get_color(&local_reg_copy));
    }
}

impl Debug for OpNode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.reg_color {
            Some(color) => {
                write!(f, " [shape=record style=filled fillcolor={} label=\"{{ ", color.to_string());
                let mut first = true;
                for inst in &self.inst {
                    if first {
                        write!(f, "({}): {}",
                        inst.borrow().get_inst_num(),
                        inst.borrow().to_string()
                        );
                        first = false;
                    } else {
                        write!(f, "\\l ({}): {}",
                               inst.borrow().get_inst_num(),
                               inst.borrow().to_string()
                        );
                    }
                }
                write!(f,  "}}\"]")
            },
            None => {
                write!(f, " [shape=record label=\"{{ ");
                let mut first = true;
                for inst in &self.inst {
                    if first {
                        write!(f, "({}): {}",
                               inst.borrow().get_inst_num(),
                               inst.borrow().to_string()
                        );
                        first = false;
                    } else {
                        write!(f, "\\l ({}): {}",
                               inst.borrow().get_inst_num(),
                               inst.borrow().to_string()
                        );
                    }
                }
                write!(f,  "}}\"]")
            }
        }
    }
}

pub struct RecurseTraverse {
    current_node: NodeIndex,
    interference_graph: Graph<OpNode, String, Directed, u32>,
    temp_val_manager: TempValManager,
    inst_node_map: HashMap<usize, NodeIndex>,
    live_inst_map: HashMap<usize, NodeIndex>,
    coalescence_map: HashMap<NodeIndex, (Option<NodeIndex>,Option<NodeIndex>)>,
    dominators: Dominators<NodeIndex>,
    while_bp: Option<NodeIndex>,
    if_bp: Option<NodeIndex>,
}

impl RecurseTraverse {
    pub fn new(current_node: NodeIndex, temp_val_manager: & TempValManager, dominators: Dominators<NodeIndex>) -> Self {
        RecurseTraverse {
            current_node,
            interference_graph: Graph::new(),
            temp_val_manager: temp_val_manager.clone(),
            inst_node_map: HashMap::new(),
            live_inst_map: HashMap::new(),
            coalescence_map: HashMap::new(),
            dominators,
            while_bp: None,
            if_bp: None,
        }
    }

    pub fn get_interference_graph(self) -> Graph<OpNode, String, Directed, u32> {
        self.interference_graph
    }

    /// Recursing options:
    /// 0-Parents                    -> End of recursion, return completed live range
    /// 1-Parent                     -> Continue directly up graph
    /// 2-Parents (neither dominate) -> An if branch. go up left (until loop header is found), then traverse up right.
    ///                                 Continue with the loop header after joining live range from left and right.
    /// 2-Parents (one dominates)    -> An else branch, loop through the branch dominated by loop twice, then traverse
    ///                                 up the dominating path.
    pub fn recursive_traversal(&mut self, irgm: &mut IRGraphManager) {
        if let Some(node_id) = self.if_bp {
            if self.current_node == node_id.clone() {
                return;
            }
        }

        let mut parents = Vec::new();
        for parent_id in irgm
            .clone()
            .graph_manager()
            .get_ref_graph()
            .neighbors_directed(self.current_node.clone(), Incoming)
        {
            parents.push(parent_id);
        }

        match parents.len() {
            0 => {
                // Final node, perform any required actions then simply return.
                self.grab_live_ranges(irgm, BlockType::standard);
                return;
            }
            1 => {
                self.grab_live_ranges(irgm, BlockType::standard);
                self.current_node = parents.pop().unwrap();
                self.recursive_traversal(irgm);
                return;
            }
            2 => {
                // two cases here, if or while
                let mut ordered_parents = Vec::new();
                let mut is_while = false;

                // This gives both information as to which control flow type it
                // is, as well as sorting for the while case.
                for node_id in parents.iter() {
                    if self.dominators.immediate_dominator(self.current_node)
                        == Some(node_id.clone())
                    {
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
                            return;
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
                    return;
                } else {

                    // This is the if case. Traverse up both paths until the dominator is hit, then return
                    // and merge the two live ranges and go through the dominating path.
                    let immediate_dominator = self
                        .dominators
                        .immediate_dominator(self.current_node)
                        .expect(
                            &format!("No dominating path found for: {:?}", self.current_node)[..],
                        )
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

                    self.recursive_traversal(irgm);

                    return;
                }
            }
            _ => {
                panic!("Should be no more than 2 parents for any given node of the graph.");
            }
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

            let inst_node;
            match self.inst_node_map.get(&inst_id) {
                Some(node_id) => {
                    inst_node = Some(node_id.clone());
                },
                None => {
                    inst_node = None;
                }
            }
            let mut is_phi = false;
            let mut x_index = None;
            let mut y_index = None;

            if inst_type == InstTy::phi {
                is_phi = true;
            }

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
                        let inst_node_id;
                        if !self.inst_node_map.contains_key(&x_inst_id) {
                            // This instruction is not already part of the live range.
                            // Create new node and add to the graph.
                            let weight = self.temp_val_manager
                                .borrow_mut_inst(&x_inst_id)
                                .borrow()
                                .active_uses()
                                .len();
                            let op_node = OpNode::new(Rc::clone(x_inst), weight * 2);
                            inst_node_id = self.interference_graph.add_node(op_node);
                            self.inst_node_map.insert(x_inst_id, inst_node_id.clone());
                        } else {
                            inst_node_id = self.inst_node_map.get(&x_inst_id).unwrap().clone();
                        }

                        if let Some(while_node) = self.while_bp.clone() {
                            // if this is true it indicates being in a while loop
                            // add 10 every time a value gets called in a loop
                            self.interference_graph
                                .node_weight_mut(inst_node_id)
                                .unwrap()
                                .add_weight(10);
                        }

                        if inst_type == InstTy::phi {
                            x_index = Some(inst_node_id.clone());
                            if block_type == BlockType::while_loop
                                || block_type == BlockType::if_phi_left
                            {
                                // Make an edge between all nodes currently in live range, then add to live range
                                for (_, node_id) in self.live_inst_map.iter() {
                                    if None
                                        == self
                                            .interference_graph
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
                                    == self
                                        .interference_graph
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
                        let inst_node_id;
                        if !self.inst_node_map.contains_key(&y_inst_id) {
                            // This instruction is not already part of the live range.
                            // Create new node and add to the graph.
                            let weight = self.temp_val_manager
                                .borrow_mut_inst(&y_inst_id)
                                .borrow()
                                .active_uses()
                                .len();
                            let op_node = OpNode::new(Rc::clone(y_inst), weight * 2);
                            inst_node_id = self.interference_graph.add_node(op_node);
                            self.inst_node_map.insert(y_inst_id, inst_node_id.clone());
                        } else {
                            inst_node_id = self.inst_node_map.get(&y_inst_id).unwrap().clone();
                        }

                        if let Some(while_node) = self.while_bp.clone() {
                            // if this is true it indicates being in a while loop
                            // add 10 every time a value gets called in a loop
                            self.interference_graph
                                .node_weight_mut(inst_node_id)
                                .unwrap()
                                .add_weight(10);
                        }

                        if inst_type == InstTy::phi {
                            y_index = Some(inst_node_id.clone());
                            if block_type == BlockType::if_phi_right
                                || block_type == BlockType::while_cont
                            {
                                // Make an edge between all nodes currently in live range, then add to live range
                                for (_, node_id) in self.live_inst_map.iter() {
                                    if None
                                        == self
                                            .interference_graph
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
                                    == self
                                        .interference_graph
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

            match inst_node {
                Some(node_id) => {
                    if !self.coalescence_map.contains_key(&node_id) {
                        self.coalescence_map.insert(node_id.clone(), (x_index,y_index));
                    }
                },
                None => {
                    // Still not added to the graph, so dont do anything.
                }
            }
        }

        /*println!(
            "Live range ending at node {} [Type: {:?}]:",
            irgm.graph_manager()
                .get_ref_graph()
                .node_weight(self.current_node)
                .unwrap()
                .get_node_id(),
            block_type
        );*/
        /*let mut live_range_str = String::new();
        for (_, live_inst) in self.live_inst_map.iter() {
            if live_range_str.is_empty() {
                live_range_str += &String::from(format!(
                    "[ {:?}",
                    self.interference_graph
                        .node_weight(live_inst.clone())
                        .unwrap()
                        .get_inst_ref()
                        .borrow()
                        .get_inst_num()
                ));
            } else {
                live_range_str += &String::from(format!(
                    ", {:?}",
                    self.interference_graph
                        .node_weight(live_inst.clone())
                        .unwrap()
                        .get_inst_ref()
                        .borrow()
                        .get_inst_num()
                ));
            }
        }
        if !live_range_str.is_empty() {
            live_range_str += " ]";
        }

        println!("{}", live_range_str);*/
    }

    pub fn coalesce_phis(&mut self) {
        let mut walkable_graph = self.interference_graph.clone();

        let mut values_to_coalesce = self.coalescence_map
            .clone()
            .iter()
            .filter(|(
                         phi_node_id, (
                            x_node_id,
                            y_node_id)
                     )|
                {
                    // Check if there is an x node first
                    if let Some(x_id) = x_node_id {
                        // Check if there is an edge between x and phi
                        if let Some(edge) = self.interference_graph
                            .find_edge_undirected(*phi_node_id.clone(),x_id.clone()) {
                            return false
                        }

                        // Check if there is a y_node
                        if let Some(y_id) = y_node_id {
                            // Check if there is an edge between x and y
                            if let Some(edge) = self.interference_graph
                                .find_edge_undirected(x_id.clone(),y_id.clone()) {
                                return false
                            }
                        }
                    }

                    // In case there is no x_node, run check for y_node again
                    if let Some(y_id) = y_node_id {
                        // Check for edge between phi and y
                        if let Some(edge) = self.interference_graph
                            .find_edge_undirected(*phi_node_id.clone(), y_id.clone()) {
                            return false
                        }
                    }

                    true
            }).map(|(op_node,(x_option,y_option))| {
            (op_node.clone(),(x_option.clone(),y_option.clone()))
        }).collect::<Vec<(NodeIndex,(Option<NodeIndex>,Option<NodeIndex>))>>();

        for (phi_id, (x_op_id, y_op_id)) in values_to_coalesce {
            if let Some(x_id) = x_op_id {
                let x_op = self.interference_graph.node_weight(x_id.clone()).unwrap().get_inst_ref()[0].clone();
                self.interference_graph.node_weight_mut(phi_id).unwrap().coalesce_inst(x_op);

                let mut neighbor_walker = self.interference_graph.neighbors_undirected(x_id.clone()).detach();

                while let Some(neighbor_id) = neighbor_walker.next_node(&walkable_graph) {
                    if let Some(edge) = self.interference_graph
                        .find_edge_undirected(phi_id,neighbor_id) {
                        // Dont add any edges
                    } else {
                        // No edge was found, add an edge
                        self.interference_graph.update_edge(phi_id, neighbor_id, String::from("black"));
                    }
                }
            }

            if let Some(y_id) = y_op_id {
                let y_op = self.interference_graph.node_weight(y_id.clone()).unwrap().get_inst_ref()[0].clone();
                self.interference_graph.node_weight_mut(phi_id).unwrap().coalesce_inst(y_op);

                let mut neighbor_walker = self.interference_graph.neighbors_undirected(y_id.clone()).detach();

                while let Some(neighbor_id) = neighbor_walker.next_node(&walkable_graph) {
                    if let Some(edge) = self.interference_graph
                        .find_edge_undirected(phi_id,neighbor_id) {
                        // Dont add any edges
                    } else {
                        // No edge was found, add an edge
                        self.interference_graph.update_edge(phi_id, neighbor_id, String::from("black"));
                    }
                }
            }
        }
    }
}

#[derive(PartialEq, Debug)]
enum BlockType {
    standard,
    while_loop,
    while_cont,
    if_phi_left,
    if_phi_right,
}
