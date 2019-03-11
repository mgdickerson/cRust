use super::{Graph, IRGraphManager, InstTy, Node, Op, TempValManager, ValTy, Value};
use lib::Optimizer::operator_dominator::OpDomHandler;
use lib::Graph::node::NodeType;

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
                        & dom_space
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
    dominators: Dominators<NodeIndex>,
    // For the phis on this one, just go through all the loop heads and add a kill.
    while_bp: Vec<NodeIndex>,
    while_kill_prop: Vec<NodeIndex>,
    if_bp: Vec<NodeIndex>,
    // This will keep track of the nodes that need a phi propagated to them
    if_kill_prop: Vec<NodeIndex>,
    propagate_kill: bool,
}

impl CLE {
    pub fn new(root_node: & NodeIndex, dominators: Dominators<NodeIndex>) -> Self {
        CLE {
            starting_node: root_node.clone(),
            current_node: root_node.clone(),
            dominators,
            while_bp: Vec::new(),
            while_kill_prop: Vec::new(),
            if_bp: Vec::new(),
            if_kill_prop: Vec::new(),
            propagate_kill: false,
        }
    }

    pub fn remove_loads(&mut self, irgm: &mut IRGraphManager) {
        // First make a traversal over the graph
        let mut visitor = irgm.graph_manager().graph_visitor(self.starting_node);
        println!("Visit Path:\n{:?}", visitor);

        for node_id in visitor.clone().iter() {
            self.current_node = node_id.clone();

            // Get the node type.
            let node_type = irgm.graph_manager()
                .get_ref_graph()
                .node_weight(self.current_node)
                .unwrap()
                .get_node_type();


        }



    }

    fn recurse_insert_kills(&mut self, irgm: &mut IRGraphManager) {
        let children = irgm.graph_manager().get_ref_graph().neighbors_directed(self.current_node, Outgoing).detach();

        let node_type = irgm.graph_manager().get_ref_graph().node_weight(self.current_node).unwrap().get_node_type();

        match node_type {
            NodeType::while_loop_header => {},
            NodeType::if_header => {},
            NodeType::phi_node => {},
            _ => {},
        }

        // Do something for phi node case inside of loop.
    }

    fn search_kill_inst(&mut self, irgm: &mut IRGraphManager) {
        let inst_list = irgm.graph_manager().get_ref_graph()
            .node_weight(self.current_node)
            .unwrap()
            .get_data_ref()
            .get_inst_list_ref()
            .clone();
        for (position, inst) in inst_list
            .iter()
            .enumerate()
            {
                let inst_ty = inst.borrow().inst_type().clone();
                if InstTy::store == inst_ty {
                    let block_id = inst.borrow().get_inst_block();
                    let kill_inst = irgm.build_op_in_block(InstTy::kill, block_id);
                    irgm.graph_manager().insert_instruction_in_node(position + 1, kill_inst, &self.current_node);

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
}