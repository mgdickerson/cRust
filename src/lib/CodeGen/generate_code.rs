// Traverse through the graph in correct traversal order
// and put all instructions in order into a single vector.
use lib::IR::ir_manager::IRGraphManager;
use lib::Graph::node::{NodeType,Node};
use lib::IR::ir::{InstTy,ValTy,Value};

use petgraph::prelude::{NodeIndex};
use petgraph::graph::Graph;
use petgraph::{Directed, Incoming, Outgoing};
use std::collections::HashMap;

pub struct CodeGen {
    irgm: IRGraphManager,
    walkable_graph: Graph<Node, String, Directed, u32>,
    traversal_path: Vec<NodeIndex>,
    reg_list: HashMap<usize, usize>,
}

impl CodeGen {
    pub fn new(irgm: IRGraphManager, reg_list: HashMap<usize, usize>) -> Self {
        let walkable_graph = irgm.graph_manager_ref().get_ref_graph().clone();

        CodeGen {
            irgm,
            walkable_graph,
            traversal_path: Vec::new(),
            reg_list
        }
    }

    pub fn get_irgm(self) -> IRGraphManager {
        self.irgm
    }

    pub fn clean_inst(&mut self) {
        let starting_node = self.irgm.graph_manager_ref().get_main_node();
        self.get_current_traversal(starting_node);
        self.clean_nodes();

        let func_list = self.irgm.function_manager().list_functions();
        for (func_name, starting_node) in func_list.iter() {
            self.get_current_traversal(starting_node.clone());
            self.clean_nodes();
        }
    }

    pub fn clean_nodes(&mut self) {
        self.irgm.address_manager().set_variable_assignments();
        let addr_manager = self.irgm.address_manager().clone();

        for node_id in self.traversal_path.iter() {
            let mut inst_list = self.irgm
                .graph_manager()
                .get_mut_ref_graph()
                .node_weight_mut(node_id.clone())
                .unwrap()
                .get_mut_data_ref()
                .get_mut_inst_list_ref()
                .clone();

            for (position, inst) in inst_list.iter_mut().enumerate() {
                inst.borrow_mut().op_to_register(&self.reg_list);
                inst.borrow_mut().address_to_const(&addr_manager);

                let inst_type = inst.borrow().inst_type().clone();
                match inst_type {
                    InstTy::bra | InstTy::bne |
                    InstTy::beq | InstTy::ble |
                    InstTy::blt | InstTy::bge |
                    InstTy::bgt => {
                        let y_val = inst.borrow().clone_y_val().unwrap();
                        if let ValTy::node_id(branch_id) = y_val.get_value().clone() {
                            let adjusted_id = self.irgm.graph_manager_ref().block_node_map().get(&branch_id.index()).unwrap().clone();
                            // This works.
                            //println!("Adjusted branch node: {}", self.irgm.graph_manager_ref().get_ref_graph().node_weight(adjusted_id).unwrap().get_node_id());

                            let mut current_search_branch = adjusted_id;
                            let mut searching = true;
                            while searching {
                                let inst_search_list = self.irgm
                                    .graph_manager_ref()
                                    .get_ref_graph()
                                    .node_weight(current_search_branch)
                                    .unwrap()
                                    .get_data_ref()
                                    .get_inst_list_ref()
                                    .clone();

                                if !inst_search_list.is_empty() {
                                    let first_inst = inst_search_list.get(0).unwrap();
                                    inst.borrow_mut().update_y_val(Value::new(ValTy::op(first_inst.clone())));
                                    searching = false;
                                } else {
                                    let mut children = self.irgm
                                        .graph_manager_ref()
                                        .get_ref_graph()
                                        .neighbors_directed(current_search_branch, Outgoing)
                                        .detach();

                                    current_search_branch = children.next_node(&self.walkable_graph).unwrap();

                                    if NodeType::exit == self.irgm
                                        .graph_manager_ref()
                                        .get_ref_graph()
                                        .node_weight(current_search_branch)
                                        .unwrap()
                                        .get_node_type() {
                                        current_search_branch = children.next_node(&self.walkable_graph).unwrap();
                                    }
                                }
                            }
                            // TODO : Should also add instruction vec with numbering of instruction (for size and position and all that)
                        }
                    },
                    _ => {},
                }
            }
        }
    }

    pub fn get_current_traversal(&mut self, starting_node: NodeIndex) {
        let mut traversal = Vec::new();

        traversal_path(&mut self.irgm, &self.walkable_graph, starting_node, &mut traversal);

        self.traversal_path = traversal;
    }
}

pub fn traversal_path(irgm: &mut IRGraphManager,
                  walkable_graph: &Graph<Node, String, Directed, u32>,
                  current_node: NodeIndex,
                  visited: &mut Vec<NodeIndex>) {
    if visited.contains(&current_node) {
        return
    }

    let mut children = irgm.graph_manager()
        .get_ref_graph()
        .neighbors_directed(current_node.clone(), Outgoing)
        .detach();

    let node_type = irgm.graph_manager()
        .get_ref_graph()
        .node_weight(current_node)
        .unwrap()
        .get_node_type();

    match node_type {
        NodeType::while_loop_header => {
            visited.push(current_node);

            let mut loop_node = current_node.clone();
            let mut bra_node = current_node.clone();
            while let Some(child_node_id) = children.next_node(walkable_graph) {
                match irgm.graph_manager_ref()
                    .get_ref_graph()
                    .node_weight(child_node_id.clone())
                    .unwrap()
                    .get_node_type() {
                    NodeType::while_node => {
                        loop_node = child_node_id;
                    }
                    NodeType::bra_node => {
                        bra_node = child_node_id;
                    }
                    NodeType::exit => {
                        // This is an exit, likely due to a removed path, just give it the exit
                        bra_node = child_node_id;
                    }
                    _ => {
                        // Probably panic here?
                        panic!("Probably should not reach this.");
                    }
                }
            }

            traversal_path(irgm,walkable_graph,loop_node,visited);
            traversal_path(irgm,walkable_graph,bra_node,visited);
        },
        NodeType::if_header => {
            visited.push(current_node.clone());

            let mut if_bra = current_node.clone();
            let mut else_bra = current_node.clone();
            while let Some(next_node_id) = children.next_node(walkable_graph) {
                match irgm
                    .graph_manager_ref()
                    .get_ref_graph()
                    .node_weight(next_node_id.clone())
                    .unwrap()
                    .get_node_type() {
                    NodeType::if_node => {
                        if_bra = next_node_id;
                    },
                    NodeType::else_node => {
                        else_bra = next_node_id;
                    },
                    NodeType::phi_node => {
                        else_bra = next_node_id;
                    },
                    _ => {},
                }
            }

            traversal_path(irgm,walkable_graph,if_bra,visited);
            traversal_path(irgm,walkable_graph,else_bra,visited);
        },
        NodeType::phi_node => {
            // Check the parents of the phi node, if both have
            // been visited, continue on this one, otherwise
            // return.
            let mut parents = irgm
                .graph_manager_ref()
                .get_ref_graph()
                .neighbors_directed(current_node.clone(), Incoming)
                .detach();

            while let Some(parent_id) = parents.next_node(walkable_graph) {
                if !visited.contains(&parent_id) {
                    return
                }
            }

            // If this point is reached, it means all parent
            // nodes have been traversed. Continue through.

            visited.push(current_node.clone());

            if let Some(child_node) = children.next_node(walkable_graph) {
                traversal_path(irgm, walkable_graph, child_node, visited);
            }
        },
        node => {
            visited.push(current_node.clone());

            if let Some(child_node) = children.next_node(walkable_graph) {
                traversal_path(irgm, walkable_graph, child_node, visited);
            }

            if let Some(error_child) = children.next_node(walkable_graph) {
                panic!("Second child found in unexpected path.");
            }
        },
    }

}