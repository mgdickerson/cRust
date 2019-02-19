use super::{Graph,GraphManager,Value,ValTy,InstTy, Node, TempValManager};
use super::IRGraphManager;
use std::collections::HashMap;
use petgraph::prelude::NodeIndex;
use lib::Lexer::token::TokenType::Var;

pub fn eval_program_constants(irgm: &mut IRGraphManager, temp_manager: &mut TempValManager) {
    // Get mutable reference to the graph manager
    let mut graph = irgm.graph_manager();

    // Get traversal order from temp_manager
    let traversal_order = temp_manager.clone_visit_order();

    // For removing instructions, the plan is to use the number of uses in temp_manager.
    // For functions that are not vital to control flow or operation, once the
    // number of uses drops to 0, it will be marked no longer in use and then
    // cleaned up by a cleaner function.

    let mut instruction_replacement_map : HashMap<usize, Value> = HashMap::new();

    for node in traversal_order.iter() {
        for inst in graph.get_mut_ref_graph().node_weight_mut(node.clone()).unwrap().get_mut_data_ref().get_inst_list_ref().iter() {
            let inst_id = inst.borrow().get_inst_num();
            let inst_ty = inst.borrow().inst_type().clone();

            // Only some of the instructions are affected by constants being evaluated.
            // Main ones that need to be addressed are (add, sub, mul, div, cmp -> bra),
            // where branch is only affected if the cmp is removed from the evaluation.
            // While Phi could change, Phis will be handled by the cleanup function.

            match inst_ty {
                InstTy::add => {
                    // TODO : This.

                },
                InstTy::sub => {
                    // TODO : This.

                },
                InstTy::mul => {
                    // TODO : This.

                },
                InstTy::div => {
                    // TODO : This.

                },
                InstTy::cmp => {
                    // TODO : This.

                },
                _ => {
                    // Nothing to do here, these are all the unaffected instructions
                },
            }
        }
    }



}

pub struct ConstEval {
    replacement_map: HashMap<usize, Value>,
    visited: Vec<usize>,
    inst_remove: Vec<usize>,
}

impl ConstEval {
    pub fn new(main_node: NodeIndex) -> Self {
        ConstEval { replacement_map: HashMap::new(), visited: Vec::new(), inst_remove: Vec::new() }
    }

    pub fn recurse_graph(&mut self, index: NodeIndex, graph_manager: &mut GraphManager) {
        // solve cosnt_expr for current node
        if let Some(node_weight) = graph_manager.get_mut_ref_graph().node_weight_mut(index.clone()) {
            self.visited.push(node_weight.get_node_id());
            //println!("Node: {}", node_weight.get_node_id());

            // Perform const eval
            for inst in node_weight.get_mut_data_ref().get_mut_inst_list_ref() {
                let inst_id = inst.borrow().get_inst_num();
                let inst_ty = inst.borrow().inst_type().clone();

                // TODO : Continue with performing constant eval
            }
        }

        // check child nodes recursively
        for child in graph_manager.get_mut_ref_graph().clone().neighbors(index) {
            if self.visited.contains(&graph_manager.get_node_id(child)) {
                continue
            }
            self.recurse_graph(child, graph_manager);
        }
    }
}

    // Walk through the graph and start solving out constant expressions
    /*for graph_node in graph.node_weights_mut() {
        // grab each instruction individually
        for inst in graph_node.get_mut_data_ref().get_mut_ref() {
            let inst_id = inst.borrow().get_inst_num();
            let inst_ty = inst.borrow().inst_type().clone();

            match inst_ty {
                InstTy::add | InstTy::sub |
                InstTy::mul | InstTy::div => {
                    // These are all the simple math instructions,
                    // first update any previously changed values.
                    let (x_val, y_val, _) = inst.borrow().get_values();
                    if let ValTy::op(op) = x_val.unwrap().get_value() {
                        if let Some(replacement_val) =
                        instruction_replacement_map.get(&op.borrow().get_inst_num())
                        {
                            inst.borrow_mut().update_x_val(replacement_val.clone());
                        }
                    }
                    if let ValTy::op(op) = y_val.unwrap().get_value() {
                        if let Some(replacement_val) =
                        instruction_replacement_map.get(&op.borrow().get_inst_num())
                        {
                            inst.borrow_mut().update_y_val(replacement_val.clone());
                        }
                    }

                    // After changing previously updated values, perform eval
                    let (x_val, y_val, _) = inst.borrow().get_values();
                    if let ValTy::con(x) = x_val.unwrap().get_value() {
                        if let ValTy::con(y) = y_val.unwrap().get_value() {

                        }
                    }
                },
                _ => {},
            }
        }
    }*/