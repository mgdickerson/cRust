use super::{GraphManager,Value,ValTy,InstTy};
use super::IRGraphManager;
use std::collections::HashMap;

pub fn const_eval(graph_manager: &mut GraphManager) {
    let mut graph = graph_manager.get_mut_ref_graph();
    let mut instruction_replacement_map : HashMap<usize, Value> = HashMap::new();
    let mut remove_vec : Vec<usize> = Vec::new();

    // Walk through the graph and start solving out constant expressions
    for graph_node in graph.node_weights_mut() {
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
    }
}