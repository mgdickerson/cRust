use super::{Graph, IRGraphManager, InstTy, Node, Op, TempValManager, ValTy, Value};
use lib::Graph::node::NodeType;
use lib::Optimizer::operator_dominator::OpDomHandler;

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use petgraph::algo::dominators::simple_fast;
use petgraph::algo::dominators::Dominators;
use petgraph::prelude::NodeIndex;
use petgraph::{Directed, Incoming, Outgoing};

pub fn trace_common_expression(
    irgm: &mut IRGraphManager,
    temp_manager: &mut TempValManager,
    root_node: NodeIndex,
) {
    // Make hashmap of function replacements
    //let inst_replacement = HashMap::new();

    // visit_order
    let visitor = irgm.graph_manager().graph_visitor(root_node.clone());
    let walkable_graph = irgm.graph_manager().get_ref_graph().clone();

    // Make quick dominance graph.
    let mut dom_graph = irgm.graph_manager().get_ref_graph().clone();
    let dom_space = simple_fast(&dom_graph, root_node.clone());
    //println!("{:?}", dom_space);

    let mut local_op_handler = OpDomHandler::new();

    for node_id in visitor {
        for inst in irgm
            .graph_manager()
            .get_mut_ref_graph()
            .node_weight_mut(node_id.clone())
            .unwrap()
            .get_mut_data_ref()
            .get_inst_list_ref()
            .iter()
        {
            let inst_ty = inst.borrow().inst_type().clone();
            let inst_id = inst.borrow().get_inst_num();
            //println!("Checking Instruction: {}", inst_id);

            match inst_ty {
                InstTy::add | InstTy::sub | InstTy::mul | InstTy::div | InstTy::phi => {
                    let (is_uniq, replacement_inst) = local_op_handler.search_or_add_inst(
                        inst.clone(),
                        node_id.clone(),
                        &dom_space,
                    );

                    if !is_uniq {
                        // This is a very good testing output.
                        //println!("Operator to be replaced. {:?} -> {:?}", inst.clone(), replacement_inst);
                        let active_uses = temp_manager
                            .borrow_mut_inst(&inst_id)
                            .borrow()
                            .active_uses()
                            .iter()
                            .map(|temp_val| temp_val.borrow().inst_val())
                            .collect::<Vec<Rc<RefCell<Op>>>>();
                        for op in active_uses {
                            // First clean up the old Phi value at instruction site
                            let replacement_value = Value::new(ValTy::op(replacement_inst.clone()));
                            op.borrow_mut()
                                .op_cleanup(inst_id.clone(), replacement_value);

                            // Get instruction id
                            let op_id = op.borrow().get_inst_num();
                            // Get inst value ref to add to y_inst temp
                            let op_temp = temp_manager.borrow_inst(&op_id).clone();

                            // Add new use to value used to replace.
                            let replacement_id = replacement_inst.borrow().get_inst_num();
                            let temp_val = temp_manager.borrow_mut_inst(&replacement_id);
                            temp_val.borrow_mut().add_use(op_temp);
                        }
                        temp_manager
                            .borrow_mut_inst(&inst_id)
                            .borrow_mut()
                            .deactivate_instruction();
                        temp_manager.clean_instruction_uses(&inst_id);
                    }
                }
                _ => {
                    // Do nothing.
                }
            }
        }
    }
}

pub struct CLE {
    starting_node: NodeIndex,
    current_node: NodeIndex,
    walkable_graph: Graph<Node, String, Directed, u32>,
    dominators: Dominators<NodeIndex>,
    // For the phis on this one, just go through all the loop heads and add a kill.
    while_bp: Vec<NodeIndex>,
    while_kill_prop: Vec<NodeIndex>,
    if_bp: Vec<NodeIndex>,
    // This will keep track of the nodes that need a phi propagated to them
    if_kill_prop: Vec<NodeIndex>,
    if_to_phi_map: HashMap<NodeIndex, NodeIndex>,
    propagate_kill: bool,
    load_map: HashMap<usize, Rc<RefCell<Op>>>,
}

impl CLE {
    pub fn new(
        irgm: &mut IRGraphManager,
        root_node: &NodeIndex,
        dominators: Dominators<NodeIndex>,
    ) -> Self {
        CLE {
            starting_node: root_node.clone(),
            current_node: root_node.clone(),
            walkable_graph: irgm.graph_manager().get_ref_graph().clone(),
            dominators,
            while_bp: Vec::new(),
            while_kill_prop: Vec::new(),
            if_bp: Vec::new(),
            if_kill_prop: Vec::new(),
            if_to_phi_map: HashMap::new(),
            propagate_kill: false,
            load_map: HashMap::new(),
        }
    }

    pub fn remove_loads(&mut self, irgm: &mut IRGraphManager, temp_manager: &mut TempValManager) {
        // First make a traversal over the graph
        self.recurse_insert_kills(irgm);

        // Now add kills to indicated nodes
        self.add_kills(irgm);

        //temp_manager.pull_temp_values(irgm.graph_manager_ref(), self.starting_node);

        let mut op_dom_handler = OpDomHandler::new();

        self.current_node = self.starting_node.clone();
        self.while_bp = Vec::new();
        self.if_bp = Vec::new();
        self.recurse_remove_loads(irgm, &mut op_dom_handler, temp_manager);
    }

    fn recurse_remove_loads(
        &mut self,
        irgm: &mut IRGraphManager,
        op_dom_handler: &mut OpDomHandler,
        temp_manager: &mut TempValManager,
    ) {
        let mut children = irgm
            .graph_manager()
            .get_ref_graph()
            .neighbors_directed(self.current_node, Outgoing)
            .detach();

        let node_id = irgm
            .graph_manager()
            .get_ref_graph()
            .node_weight(self.current_node)
            .unwrap()
            .get_node_id();
        let node_type = irgm
            .graph_manager()
            .get_ref_graph()
            .node_weight(self.current_node)
            .unwrap()
            .get_node_type();

        match node_type {
            NodeType::while_loop_header => {
                if self.while_bp.contains(&self.current_node) {
                    return;
                }

                self.remove_common_loads(irgm, op_dom_handler, temp_manager);

                self.while_bp.push(self.current_node.clone());

                let mut loop_node = self.current_node.clone();
                let mut bra_node = self.current_node.clone();
                while let Some(next_node_id) = children.next_node(&self.walkable_graph) {
                    match irgm
                        .graph_manager()
                        .get_ref_graph()
                        .node_weight(next_node_id.clone())
                        .unwrap()
                        .get_node_type()
                    {
                        NodeType::while_node => {
                            loop_node = next_node_id;
                        }
                        NodeType::bra_node => {
                            bra_node = next_node_id;
                        }
                        NodeType::exit => {
                            // This is an exit, likely due to a removed path, just give it the exit
                            bra_node = next_node_id;
                        }
                        _ => {
                            // Probably panic here?
                            panic!("Probably should not reach this.");
                        }
                    }
                }

                // Make some save points. Each branch will get a clone of the current op_dom
                let local_current = self.current_node.clone();
                let local_op_dom = op_dom_handler.clone();

                // First recurse through the loop of the while
                self.current_node = loop_node;
                self.recurse_remove_loads(irgm, &mut local_op_dom.clone(), temp_manager);

                // Once this node has been reached again, we can return to branch side
                // First remove this node from while bp
                let removed_item = self.while_bp.pop();
                if Some(local_current) == removed_item {
                    // All is well
                } else {
                    // Some error occured, perhaps wrong item popped.
                    panic!("Popped wrong item for while loop.");
                }

                self.current_node = bra_node;
                self.recurse_remove_loads(irgm, &mut local_op_dom.clone(), temp_manager);
            }
            NodeType::if_header => {
                self.remove_common_loads(irgm, op_dom_handler, temp_manager);

                self.if_bp.push(self.current_node.clone());

                // sort nodes
                let mut if_bra = self.current_node.clone();
                let mut else_bra = self.current_node.clone();
                while let Some(next_node_id) = children.next_node(&self.walkable_graph) {
                    match irgm
                        .graph_manager()
                        .get_ref_graph()
                        .node_weight(next_node_id.clone())
                        .unwrap()
                        .get_node_type()
                    {
                        NodeType::if_node => {
                            if_bra = next_node_id;
                        }
                        NodeType::else_node => {
                            else_bra = next_node_id;
                        }
                        NodeType::phi_node => {
                            else_bra = next_node_id;
                        }
                        _ => {
                            // Do nothing here
                        }
                    }
                }

                let local_current = self.current_node.clone();
                let mut if_op_dom = op_dom_handler.clone();
                let mut else_op_dom = op_dom_handler.clone();

                // First check if_branch is still valid
                if if_bra != local_current {
                    // If branch is a valid and unique branch
                    self.current_node = if_bra;
                    self.recurse_remove_loads(irgm, &mut if_op_dom, temp_manager);
                }

                if else_bra != local_current {
                    self.current_node = else_bra;
                    self.recurse_remove_loads(irgm, &mut else_op_dom, temp_manager);
                }

                // Quick sanity check to make sure we are on the phi node before continuing.
                let node_type = irgm
                    .graph_manager()
                    .get_ref_graph()
                    .node_weight(self.current_node)
                    .unwrap()
                    .get_node_type();

                if node_type == NodeType::phi_node {
                    let removed_item = self.if_bp.pop();
                    if Some(local_current) == removed_item {
                        // All is well
                    } else {
                        // Some error occured, perhaps wrong item popped.
                        panic!("Popped wrong item for if-else.");
                    }

                    // Merge the two op_doms
                    if_op_dom.merge_op_doms(&else_op_dom);

                    self.recurse_remove_loads(irgm, &mut if_op_dom, temp_manager);
                } else {
                    // I dont think this case should ever be reached
                    panic!("Reached end of if statement without a phi being reached.");
                }
            }
            NodeType::phi_node => {
                if let Some(if_node) = self.if_bp.last() {
                    if Some(if_node.clone())
                        == self
                            .dominators
                            .immediate_dominator(self.current_node.clone())
                    {
                        // Nodes match, return
                        return;
                    } else {
                        // Nodes did not match, I am assuming this is a removed constant case.
                        // Print it out just to check and be sure.
                        //println!("Immediate Dominator: {:?}", self.dominators.immediate_dominator(self.current_node.clone()));
                        return;
                    }
                }

                self.remove_common_loads(irgm, op_dom_handler, temp_manager);

                // Get new current node
                let child_id = children.next_node(&self.walkable_graph);
                if let Some(child_node_id) = child_id {
                    self.current_node = child_node_id;
                    self.recurse_remove_loads(irgm, op_dom_handler, temp_manager);
                } else {
                    // End of program.
                }
            }
            _ => {
                self.remove_common_loads(irgm, op_dom_handler, temp_manager);

                if let Some(child_node_id) = children.next_node(&self.walkable_graph) {
                    self.current_node = child_node_id;
                    self.recurse_remove_loads(irgm, op_dom_handler, temp_manager);
                }

                if let Some(error_child) = children.next_node(&self.walkable_graph) {
                    panic!("Second child found in unexpected path.");
                }

                // If this point is reached, program has successfully completed.
            }
        }
    }

    fn remove_common_loads(
        &mut self,
        irgm: &mut IRGraphManager,
        op_dom_handler: &mut OpDomHandler,
        temp_manager: &mut TempValManager,
    ) {
        let inst_list = irgm
            .graph_manager()
            .get_ref_graph()
            .node_weight(self.current_node.clone())
            .unwrap()
            .get_data_ref()
            .get_inst_list_ref()
            .clone();

        let list_follower = inst_list.clone();

        for (position, inst) in inst_list.iter().enumerate() {
            let inst_ty = inst.borrow().inst_type().clone();
            let inst_id = inst.borrow().get_inst_num();

            match inst_ty {
                InstTy::kill => {
                    // Gets killed in dce by name specifically
                    op_dom_handler.reset_op_set();
                }
                InstTy::load => {
                    let load_inst = inst.borrow().clone_y_val()
                        .expect("All valid load instructions should have valid adda or add instructions referenced");

                    if let ValTy::op(ref_op) = load_inst.get_value() {
                        let (is_uniq, replacement_inst) = op_dom_handler.search_or_add_inst(
                            ref_op.clone(),
                            self.current_node.clone(),
                            &self.dominators,
                        );

                        if !is_uniq {
                            let active_uses = temp_manager
                                .borrow_inst(&inst_id)
                                .borrow()
                                .active_uses()
                                .iter()
                                .map(|temp_val| temp_val.borrow().inst_val())
                                .collect::<Vec<Rc<RefCell<Op>>>>();

                            for op in active_uses {
                                // First clean up the old Phi value at instruction site
                                let load_op = self
                                    .load_map
                                    .get(&replacement_inst.borrow_mut().get_inst_num())
                                    .unwrap();
                                let replacement_value = Value::new(ValTy::op(load_op.clone()));
                                op.borrow_mut()
                                    .op_cleanup(inst_id.clone(), replacement_value);

                                // Get instruction id
                                let op_id = op.borrow().get_inst_num();
                                // Get inst value ref to add to y_inst temp
                                let op_temp = temp_manager.borrow_inst(&op_id).clone();

                                // Add new use to value used to replace.
                                let replacement_id = load_op.borrow().get_inst_num();
                                let temp_val = temp_manager.borrow_mut_inst(&replacement_id);
                                temp_val.borrow_mut().add_use(op_temp);
                            }

                            inst.borrow_mut().deactivate();
                            ref_op.borrow_mut().deactivate();
                        } else {
                            self.load_map
                                .insert(replacement_inst.borrow_mut().get_inst_num(), inst.clone());
                        }
                    } else {
                        // I dont think any load has a value other than an op, lets test
                        panic!("Value other than Op found in load.");
                    }
                }
                InstTy::store => {
                    // I dont think I can kill any of these?
                }
                _ => {
                    // Do nothing.
                }
            }
        }
    }

    fn recurse_insert_kills(&mut self, irgm: &mut IRGraphManager) {
        let mut children = irgm
            .graph_manager()
            .get_ref_graph()
            .neighbors_directed(self.current_node, Outgoing)
            .detach();

        let node_type = irgm
            .graph_manager()
            .get_ref_graph()
            .node_weight(self.current_node)
            .unwrap()
            .get_node_type();

        match node_type {
            NodeType::while_loop_header => {
                // If this node is reached and it is already marked in the bp, then return to call site.
                if self.while_bp.contains(&self.current_node) {
                    return;
                }

                // Search the node for kill instructions
                self.search_kill_inst(irgm);

                // Node has not been called yet add to bp
                self.while_bp.push(self.current_node.clone());

                let mut loop_node = self.current_node.clone();
                let mut bra_node = self.current_node.clone();
                while let Some(next_node_id) = children.next_node(&self.walkable_graph) {
                    match irgm
                        .graph_manager()
                        .get_ref_graph()
                        .node_weight(next_node_id.clone())
                        .unwrap()
                        .get_node_type()
                    {
                        NodeType::while_node => {
                            loop_node = next_node_id;
                        }
                        NodeType::bra_node => {
                            bra_node = next_node_id;
                        }
                        NodeType::exit => {
                            // This is an exit, likely due to a removed path, just give it the exit
                            bra_node = next_node_id;
                        }
                        _ => {
                            // Probably panic here?
                            panic!("Probably should not reach this.");
                        }
                    }
                }

                let local_current = self.current_node.clone();

                // Make some save points.

                // First recurse through the loop of the while
                self.current_node = loop_node;
                self.recurse_insert_kills(irgm);

                // Once this node has been reached again, we can return to branch side
                // First remove this node from while bp
                let removed_item = self.while_bp.pop();
                if Some(local_current) == removed_item {
                    // All is well
                } else {
                    // Some error occured, perhaps wrong item popped.
                    panic!("Popped wrong item for while loop.");
                }

                self.current_node = bra_node;
                self.recurse_insert_kills(irgm);
            }
            NodeType::if_header => {
                // first traverse the node
                self.search_kill_inst(irgm);

                // Add if to kill tracker
                self.if_bp.push(self.current_node.clone());

                // sort nodes
                let mut if_bra = self.current_node.clone();
                let mut else_bra = self.current_node.clone();
                while let Some(next_node_id) = children.next_node(&self.walkable_graph) {
                    match irgm
                        .graph_manager()
                        .get_ref_graph()
                        .node_weight(next_node_id.clone())
                        .unwrap()
                        .get_node_type()
                    {
                        NodeType::if_node => {
                            if_bra = next_node_id;
                        }
                        NodeType::else_node => {
                            else_bra = next_node_id;
                        }
                        NodeType::phi_node => {
                            else_bra = next_node_id;
                        }
                        _ => {
                            // Do nothing here
                        }
                    }
                }

                let local_current = self.current_node.clone();

                // First check if_branch is still valid
                if if_bra != local_current {
                    // If branch is a valid and unique branch
                    self.current_node = if_bra;
                    self.recurse_insert_kills(irgm);
                }

                if else_bra != local_current {
                    self.current_node = else_bra;
                    self.recurse_insert_kills(irgm);
                }

                // Quick sanity check to make sure we are on the phi node before continuing.
                let node_type = irgm
                    .graph_manager()
                    .get_ref_graph()
                    .node_weight(self.current_node)
                    .unwrap()
                    .get_node_type();

                if node_type == NodeType::phi_node {
                    let removed_item = self.if_bp.pop();
                    if Some(local_current) == removed_item {
                        // All is well
                    } else {
                        // Some error occured, perhaps wrong item popped.
                        panic!("Popped wrong item for if-else.");
                    }
                    self.if_to_phi_map
                        .insert(local_current, self.current_node.clone());

                    self.recurse_insert_kills(irgm);
                } else {
                    // I dont think this case should ever be reached
                    panic!("Reached end of if statement without a phi being reached.");
                }
            }
            NodeType::phi_node => {
                if let Some(if_node) = self.if_bp.last() {
                    if Some(if_node.clone())
                        == self
                            .dominators
                            .immediate_dominator(self.current_node.clone())
                    {
                        // Nodes match, return
                        return;
                    } else {
                        // Nodes did not match, I am assuming this is a removed constant case.
                        // Print it out just to check and be sure.
                        //println!("Immediate Dominator: {:?}", self.dominators.immediate_dominator(self.current_node.clone()));
                        return;
                    }
                }

                // If we have passed the initial check, it means we are returning
                // to this node after both if and else paths have been traversed.
                self.search_kill_inst(irgm);

                // Get new current node
                let child_id = children.next_node(&self.walkable_graph);
                if let Some(child_node_id) = child_id {
                    self.current_node = child_node_id;
                    self.recurse_insert_kills(irgm);
                } else {
                    // End of program.
                }
            }
            _ => {
                self.search_kill_inst(irgm);

                if let Some(child_node_id) = children.next_node(&self.walkable_graph) {
                    self.current_node = child_node_id;
                    self.recurse_insert_kills(irgm);
                }

                if let Some(error_child) = children.next_node(&self.walkable_graph) {
                    panic!("Second child found in unexpected path.");
                }

                // If this point is reached, program has successfully completed.
            }
        }

        // Do something for phi node case inside of loop.
    }

    fn search_kill_inst(&mut self, irgm: &mut IRGraphManager) {
        let inst_list = irgm
            .graph_manager()
            .get_ref_graph()
            .node_weight(self.current_node)
            .unwrap()
            .get_data_ref()
            .get_inst_list_ref()
            .clone();

        let mut adjustment = 1;

        for (position, inst) in inst_list.iter().enumerate() {
            let inst_ty = inst.borrow().inst_type().clone();
            if InstTy::store == inst_ty {
                let block_id = inst.borrow().get_inst_block();
                let kill_inst = irgm.build_op_in_block(InstTy::kill, block_id);
                irgm.graph_manager().insert_instruction_in_node(
                    position + adjustment.clone(),
                    kill_inst,
                    &self.current_node,
                );
                adjustment += 1;

                // A store instruction was found, if inside of any depth loop propagate the kill instruction to all loop depths.
                for node_id in self.while_bp.iter() {
                    if !self.while_kill_prop.contains(node_id) {
                        self.while_kill_prop.push(node_id.clone());
                    }
                }

                for node_id in self.if_bp.iter() {
                    if !self.if_kill_prop.contains(node_id) {
                        self.if_kill_prop.push(node_id.clone());
                    }
                }
            }
        }
    }

    pub fn add_kills(&mut self, irgm: &mut IRGraphManager) {
        // Insert kill instruction to phis
        for node_id in self.if_kill_prop.iter() {
            let phi_id = self.if_to_phi_map.get(&node_id).unwrap().clone();

            let kill_op = irgm.build_op_in_block(InstTy::kill, phi_id.index());
            irgm.graph_manager()
                .insert_instruction_in_node(0, kill_op, &phi_id);
        }

        // Insert kill instruction to while headers
        for node_id in self.while_kill_prop.iter() {
            let kill_op = irgm.build_op_in_block(InstTy::kill, node_id.index());
            irgm.graph_manager()
                .insert_instruction_in_node(0, kill_op, &node_id);
        }
    }
}
