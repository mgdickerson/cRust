use super::{Graph,GraphManager,Value,ValTy,InstTy, Node};
use super::IRGraphManager;
use std::collections::HashMap;
use petgraph::prelude::NodeIndex;
use lib::Lexer::token::TokenType::Var;

pub fn eval_program_constants(irgm: &mut IRGraphManager) {
    let main_node = irgm.graph_manager().get_main_node();
    let mut graph = irgm.graph_manager();

    let mut instruction_replacement_map : HashMap<usize, Value> = HashMap::new();
    let mut remove_vec : Vec<usize> = Vec::new();

    // Start by walking main node
    let mut main_eval = ConstEval::new(main_node);
    main_eval.recurse_graph(main_node, graph);


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
            for inst in node_weight.get_mut_data_ref().get_inst_list_ref() {
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