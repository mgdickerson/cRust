use super::{Graph,GraphManager,Value,ValTy,InstTy, Node, TempValManager};
use super::IRGraphManager;

use std::collections::HashMap;

use petgraph::prelude::NodeIndex;
use lib::Lexer::token::TokenType::Var;

pub fn eval_program_constants(irgm: &mut IRGraphManager, temp_manager: &mut TempValManager) -> Result<(), String> {
    println!("Program Eval is being called.");

    // Get mutable reference to the graph manager
    let mut graph = irgm.graph_manager();

    // Get traversal order from temp_manager
    let traversal_order = temp_manager.clone_visit_order();

    // For removing instructions, the plan is to use the number of uses in temp_manager.
    // For functions that are not vital to control flow or operation, once the
    // number of uses drops to 0, it will be marked no longer in use and then
    // cleaned up by a cleaner function.

    let mut value_sub_map : HashMap<usize, i32> = HashMap::new();
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
                    let inst_val_ty = inst.borrow().get_val_ty();
                    match inst_val_ty {
                        // x_val is const, y_val is const
                        (Some(ValTy::con(x_val)),Some(ValTy::con(y_val))) => {
                            let sum = x_val + y_val;
                            if x_val != 0 {
                                // This is not being called, but it is still good to have just in case.
                                // println!("Quick tracker to see if this ever gets called.");
                                inst.borrow_mut().update_x_val(Value::new(ValTy::con(0)));
                                inst.borrow_mut().update_y_val(Value::new(ValTy::con(sum.clone())));
                            }
                            value_sub_map.insert(inst_id, sum);
                        },
                        // x_val is const, y_val is an Op
                        (Some(ValTy::con(x_val)), Some(ValTy::op(y_op))) => {
                            let y_inst_id = y_op.borrow().get_inst_num();
                            // If the op in y_val has previously been evaluated then use the replacement map to
                            // evaluate this new value, update, and remove. Otherwise, do nothing.
                            if let Some(con_val) = value_sub_map.clone().get(&y_inst_id) {
                                let sum = x_val + con_val.clone();
                                let mut val = sum.clone();
                                if sum < 0 {
                                    val = val.abs();
                                    inst.borrow_mut().update_inst_ty(InstTy::sub);
                                }
                                inst.borrow_mut().update_x_val(Value::new(ValTy::con(0)));
                                inst.borrow_mut().update_y_val(Value::new(ValTy::con(val)));

                                // Add instruction to value_sub_map
                                value_sub_map.insert(inst_id, sum);

                                // After updating the x and y values, remove use of y_op
                                temp_manager.borrow_mut_inst(&y_inst_id).borrow_mut().remove_use(&inst_id);
                            }
                        },
                        // x_val is an Op, y_val is const
                        (Some(ValTy::op(x_op)), Some(ValTy::con(y_val))) => {
                            let x_inst_id = x_op.borrow().get_inst_num();
                            // If the op in x_val has previously been evaluated then use the replacement map to
                            // evaluate this new value, update, and remove. Otherwise, do nothing.
                            if let Some(con_val) = value_sub_map.clone().get(&x_inst_id) {
                                let sum = con_val.clone() + y_val;
                                let mut val = sum.clone();
                                if sum < 0 {
                                    val = val.abs();
                                    inst.borrow_mut().update_inst_ty(InstTy::sub);
                                }
                                inst.borrow_mut().update_x_val(Value::new(ValTy::con(0)));
                                inst.borrow_mut().update_y_val(Value::new(ValTy::con(val)));

                                // Add instruction to value_sub_map
                                value_sub_map.insert(inst_id, sum);

                                // After updating, remove use of x_op
                                temp_manager.borrow_mut_inst(&x_inst_id).borrow_mut().remove_use(&inst_id);
                            }
                        },
                        // x_val is an Op, y_val is an Op
                        (Some(ValTy::op(x_op)), Some(ValTy::op(y_op))) => {
                            let x_inst_id = x_op.borrow().get_inst_num();
                            let y_inst_id = y_op.borrow().get_inst_num();
                            // Check to see if both have been evaluated
                            match (value_sub_map.clone().get(&x_inst_id), value_sub_map.clone().get(&y_inst_id)) {
                                (Some(x_con), Some(y_con)) => {
                                    let sum = x_con.clone() + y_con.clone();
                                    let mut val = sum.clone();
                                    if sum < 0 {
                                        val = val.abs();
                                        inst.borrow_mut().update_inst_ty(InstTy::sub);
                                    }
                                    inst.borrow_mut().update_x_val(Value::new(ValTy::con(0)));
                                    inst.borrow_mut().update_y_val(Value::new(ValTy::con(val)));

                                    // Add instruction to value_sub_map
                                    value_sub_map.insert(inst_id, sum);

                                    // After updating the x and y values, remove use of x_op then y_op
                                    temp_manager.borrow_mut_inst(&x_inst_id).borrow_mut().remove_use(&inst_id);
                                    temp_manager.borrow_mut_inst(&y_inst_id).borrow_mut().remove_use(&inst_id);
                                },
                                (Some(x_con), None) => {
                                    let val;
                                    if x_con.clone() < 0 {
                                        val = x_con.abs();
                                        inst.borrow_mut().update_inst_ty(InstTy::sub);
                                    } else { val = x_con.clone(); }
                                    inst.borrow_mut().update_x_val(Value::new((ValTy::op(y_op))));
                                    inst.borrow_mut().update_y_val(Value::new(ValTy::con(val)));

                                    // Nothing to add to the value sub_map
                                    // x has been updated, so it should be removed.
                                    temp_manager.borrow_mut_inst(&x_inst_id).borrow_mut().remove_use(&inst_id);
                                },
                                (None, Some(y_con)) => {
                                    let val;
                                    if y_con.clone() < 0 {
                                        val = y_con.abs();
                                        inst.borrow_mut().update_inst_ty(InstTy::sub);
                                    } else { val = y_con.clone(); }

                                    inst.borrow_mut().update_y_val(Value::new(ValTy::con(val)));

                                    // Nothing to add to value sub_map
                                    // y has been updated, so it should be removed.
                                    temp_manager.borrow_mut_inst(&y_inst_id).borrow_mut().remove_use(&inst_id);
                                }
                                _ => {},
                            }
                        },
                        // All remaining cases
                        _ => {},
                    }
                },
                InstTy::sub => {
                    let inst_val_ty = inst.borrow().get_val_ty();
                    match inst_val_ty {
                        // x_val is const, y_val is const
                        (Some(ValTy::con(x_val)),Some(ValTy::con(y_val))) => {
                            let sum = x_val - y_val;
                            let mut val = sum.clone();
                            if x_val != 0 {
                                // This is not being called, but it is still good to have just in case.
                                // println!("Quick tracker to see if this ever gets called.");
                                if sum > 0 {
                                    inst.borrow_mut().update_inst_ty(InstTy::add);
                                } else {
                                    val = val.abs();
                                }
                                inst.borrow_mut().update_x_val(Value::new(ValTy::con(0)));
                                inst.borrow_mut().update_y_val(Value::new(ValTy::con(val)));
                            }
                            value_sub_map.insert(inst_id, sum);
                        },
                        // x_val is const, y_val is an Op
                        (Some(ValTy::con(x_val)), Some(ValTy::op(y_op))) => {
                            let y_inst_id = y_op.borrow().get_inst_num();
                            // If the op in y_val has previously been evaluated then use the replacement map to
                            // evaluate this new value, update, and remove. Otherwise, do nothing.
                            if let Some(con_val) = value_sub_map.clone().get(&y_inst_id) {
                                let sum = x_val - con_val.clone();
                                let mut val = sum.clone();
                                if sum > 0 {
                                    inst.borrow_mut().update_inst_ty(InstTy::add);
                                } else {
                                    val = val.abs();
                                }
                                inst.borrow_mut().update_x_val(Value::new(ValTy::con(0)));
                                inst.borrow_mut().update_y_val(Value::new(ValTy::con(val)));

                                // Add instruction to value_sub_map
                                value_sub_map.insert(inst_id, sum);

                                // After updating the x and y values, remove use of y_op
                                temp_manager.borrow_mut_inst(&y_inst_id).borrow_mut().remove_use(&inst_id);
                            }
                        },
                        // x_val is an Op, y_val is const
                        (Some(ValTy::op(x_op)), Some(ValTy::con(y_val))) => {
                            let x_inst_id = x_op.borrow().get_inst_num();
                            // If the op in x_val has previously been evaluated then use the replacement map to
                            // evaluate this new value, update, and remove. Otherwise, do nothing.
                            if let Some(con_val) = value_sub_map.clone().get(&x_inst_id) {
                                let sum = con_val.clone() - y_val;
                                let mut val = sum.clone();
                                if sum > 0 {
                                    inst.borrow_mut().update_inst_ty(InstTy::add);
                                } else {
                                    val = val.abs();
                                }
                                inst.borrow_mut().update_x_val(Value::new(ValTy::con(0)));
                                inst.borrow_mut().update_y_val(Value::new(ValTy::con(val)));

                                // Add instruction to value_sub_map
                                value_sub_map.insert(inst_id, sum);

                                // After updating, remove use of x_op
                                temp_manager.borrow_mut_inst(&x_inst_id).borrow_mut().remove_use(&inst_id);
                            }
                        },
                        // x_val is an Op, y_val is an Op
                        (Some(ValTy::op(x_op)), Some(ValTy::op(y_op))) => {
                            let x_inst_id = x_op.borrow().get_inst_num();
                            let y_inst_id = y_op.borrow().get_inst_num();
                            // Check to see if both have been evaluated
                            match (value_sub_map.clone().get(&x_inst_id), value_sub_map.clone().get(&y_inst_id)) {
                                (Some(x_con), Some(y_con)) => {
                                    let sum = x_con.clone() - y_con.clone();
                                    let mut val = sum.clone();
                                    if sum > 0 {
                                        inst.borrow_mut().update_inst_ty(InstTy::add);
                                    } else {
                                        val = val.abs();
                                    }
                                    inst.borrow_mut().update_x_val(Value::new(ValTy::con(0)));
                                    inst.borrow_mut().update_y_val(Value::new(ValTy::con(val)));

                                    // Add instruction to value_sub_map
                                    value_sub_map.insert(inst_id, sum);

                                    // After updating the x and y values, remove use of x_op then y_op
                                    temp_manager.borrow_mut_inst(&x_inst_id).borrow_mut().remove_use(&inst_id);
                                    temp_manager.borrow_mut_inst(&y_inst_id).borrow_mut().remove_use(&inst_id);
                                },
                                (Some(x_con), None) => {
                                    // As order matters very much for subtraction instructions I dont think I should mess with this
                                    // println!("Sub replacement hits x_con, y = none. Inst_Num: {}", inst_id);
                                },
                                (None, Some(y_con)) => {
                                    let mut val = y_con.clone();
                                    if val < 0 {
                                        val = val.abs();
                                        inst.borrow_mut().update_inst_ty(InstTy::add);
                                    }
                                    inst.borrow_mut().update_y_val(Value::new(ValTy::con(y_con.clone())));

                                    // Nothing to add to value sub_map
                                    // y has been updated, so it should be removed.
                                    temp_manager.borrow_mut_inst(&y_inst_id).borrow_mut().remove_use(&inst_id);
                                }
                                _ => {},
                            }
                        },
                        // All remaining cases
                        _ => {},
                    }
                },
                InstTy::mul => {
                    let inst_val_ty = inst.borrow().get_val_ty();
                    match inst_val_ty {
                        // x_val is const, y_val is const
                        (Some(ValTy::con(x_val)),Some(ValTy::con(y_val))) => {
                            let sum = x_val * y_val;
                            // This is not being called, but it is still good to have just in case.
                            // println!("Quick tracker to see if this ever gets called.");
                            inst.borrow_mut().update_inst_ty(InstTy::add);
                            inst.borrow_mut().update_x_val(Value::new(ValTy::con(0)));
                            inst.borrow_mut().update_y_val(Value::new(ValTy::con(sum.clone())));

                            value_sub_map.insert(inst_id, sum);
                        },
                        // x_val is const, y_val is an Op
                        (Some(ValTy::con(x_val)), Some(ValTy::op(y_op))) => {
                            let y_inst_id = y_op.borrow().get_inst_num();
                            // If the op in y_val has previously been evaluated then use the replacement map to
                            // evaluate this new value, update, and remove. Otherwise, do nothing.
                            if let Some(con_val) = value_sub_map.clone().get(&y_inst_id) {
                                let sum = x_val * con_val.clone();
                                let mut val = sum.clone();
                                if sum < 0 {
                                    val = val.abs();
                                    inst.borrow_mut().update_inst_ty(InstTy::sub);
                                } else {
                                    inst.borrow_mut().update_inst_ty(InstTy::add);
                                }
                                inst.borrow_mut().update_x_val(Value::new(ValTy::con(0)));
                                inst.borrow_mut().update_y_val(Value::new(ValTy::con(val)));

                                // Add instruction to value_sub_map
                                value_sub_map.insert(inst_id, sum);

                                // After updating the x and y values, remove use of y_op
                                temp_manager.borrow_mut_inst(&y_inst_id).borrow_mut().remove_use(&inst_id);
                            }
                        },
                        // x_val is an Op, y_val is const
                        (Some(ValTy::op(x_op)), Some(ValTy::con(y_val))) => {
                            let x_inst_id = x_op.borrow().get_inst_num();
                            // If the op in x_val has previously been evaluated then use the replacement map to
                            // evaluate this new value, update, and remove. Otherwise, do nothing.
                            if let Some(con_val) = value_sub_map.clone().get(&x_inst_id) {
                                let sum = con_val.clone() * y_val;
                                let mut val = sum.clone();
                                if sum < 0 {
                                    val = val.abs();
                                    inst.borrow_mut().update_inst_ty(InstTy::sub);
                                } else {
                                    inst.borrow_mut().update_inst_ty(InstTy::add);
                                }
                                inst.borrow_mut().update_x_val(Value::new(ValTy::con(0)));
                                inst.borrow_mut().update_y_val(Value::new(ValTy::con(val)));

                                // Add instruction to value_sub_map
                                value_sub_map.insert(inst_id, sum);

                                // After updating, remove use of x_op
                                temp_manager.borrow_mut_inst(&x_inst_id).borrow_mut().remove_use(&inst_id);
                            }
                        },
                        // x_val is an Op, y_val is an Op
                        (Some(ValTy::op(x_op)), Some(ValTy::op(y_op))) => {
                            let x_inst_id = x_op.borrow().get_inst_num();
                            let y_inst_id = y_op.borrow().get_inst_num();
                            // Check to see if both have been evaluated
                            match (value_sub_map.clone().get(&x_inst_id), value_sub_map.clone().get(&y_inst_id)) {
                                (Some(x_con), Some(y_con)) => {
                                    let sum = x_con.clone() * y_con.clone();
                                    let mut val = sum.clone();
                                    if sum < 0 {
                                        val = val.abs();
                                        inst.borrow_mut().update_inst_ty(InstTy::sub);
                                    } else {
                                        inst.borrow_mut().update_inst_ty(InstTy::add);
                                    }
                                    inst.borrow_mut().update_x_val(Value::new(ValTy::con(0)));
                                    inst.borrow_mut().update_y_val(Value::new(ValTy::con(val)));

                                    // Add instruction to value_sub_map
                                    value_sub_map.insert(inst_id, sum);

                                    // After updating the x and y values, remove use of x_op then y_op
                                    temp_manager.borrow_mut_inst(&x_inst_id).borrow_mut().remove_use(&inst_id);
                                    temp_manager.borrow_mut_inst(&y_inst_id).borrow_mut().remove_use(&inst_id);
                                },
                                (Some(x_con), None) => {
                                    // This will not resolve anyway, so nothing to change or update.
                                },
                                (None, Some(y_con)) => {
                                    // This will not resolve anyway, so nothing to change or update.
                                }
                                _ => {},
                            }
                        },
                        // All remaining cases
                        _ => {},
                    }
                },
                InstTy::div => {
                    let inst_val_ty = inst.borrow().get_val_ty();
                    match inst_val_ty {
                        // x_val is const, y_val is const
                        (Some(ValTy::con(x_val)),Some(ValTy::con(y_val))) => {
                            if y_val == 0 {
                                return Err(format!("Instruction {} attempted to divide by 0", inst_id));
                            }
                            let sum = x_val / y_val;
                            // This is not being called, but it is still good to have just in case.
                            // println!("Quick tracker to see if this ever gets called.");
                            inst.borrow_mut().update_inst_ty(InstTy::add);
                            inst.borrow_mut().update_x_val(Value::new(ValTy::con(0)));
                            inst.borrow_mut().update_y_val(Value::new(ValTy::con(sum.clone())));

                            value_sub_map.insert(inst_id, sum);
                        },
                        // x_val is const, y_val is an Op
                        (Some(ValTy::con(x_val)), Some(ValTy::op(y_op))) => {
                            let y_inst_id = y_op.borrow().get_inst_num();
                            // If the op in y_val has previously been evaluated then use the replacement map to
                            // evaluate this new value, update, and remove. Otherwise, do nothing.
                            if let Some(con_val) = value_sub_map.clone().get(&y_inst_id) {
                                if con_val.clone() == 0 {
                                    return Err(format!("Instruction {} attempted to divide by 0", inst_id));
                                }
                                let sum = x_val / con_val.clone();
                                let mut val = sum.clone();
                                if sum < 0 {
                                    val = val.abs();
                                    inst.borrow_mut().update_inst_ty(InstTy::sub);
                                } else {
                                    inst.borrow_mut().update_inst_ty(InstTy::add);
                                }
                                inst.borrow_mut().update_x_val(Value::new(ValTy::con(0)));
                                inst.borrow_mut().update_y_val(Value::new(ValTy::con(val)));

                                // Add instruction to value_sub_map
                                value_sub_map.insert(inst_id, sum);

                                // After updating the x and y values, remove use of y_op
                                temp_manager.borrow_mut_inst(&y_inst_id).borrow_mut().remove_use(&inst_id);
                            }
                        },
                        // x_val is an Op, y_val is const
                        (Some(ValTy::op(x_op)), Some(ValTy::con(y_val))) => {
                            let x_inst_id = x_op.borrow().get_inst_num();
                            // If the op in x_val has previously been evaluated then use the replacement map to
                            // evaluate this new value, update, and remove. Otherwise, do nothing.
                            if let Some(con_val) = value_sub_map.clone().get(&x_inst_id) {
                                if y_val == 0 {
                                    return Err(format!("Instruction {} attempted to divide by 0", inst_id));
                                }
                                let sum = con_val.clone() / y_val;
                                let mut val = sum.clone();
                                if sum < 0 {
                                    val = val.abs();
                                    inst.borrow_mut().update_inst_ty(InstTy::sub);
                                } else {
                                    inst.borrow_mut().update_inst_ty(InstTy::add);
                                }
                                inst.borrow_mut().update_x_val(Value::new(ValTy::con(0)));
                                inst.borrow_mut().update_y_val(Value::new(ValTy::con(val)));

                                // Add instruction to value_sub_map
                                value_sub_map.insert(inst_id, sum);

                                // After updating, remove use of x_op
                                temp_manager.borrow_mut_inst(&x_inst_id).borrow_mut().remove_use(&inst_id);
                            }
                        },
                        // x_val is an Op, y_val is an Op
                        (Some(ValTy::op(x_op)), Some(ValTy::op(y_op))) => {
                            let x_inst_id = x_op.borrow().get_inst_num();
                            let y_inst_id = y_op.borrow().get_inst_num();
                            // Check to see if both have been evaluated
                            match (value_sub_map.clone().get(&x_inst_id), value_sub_map.clone().get(&y_inst_id)) {
                                (Some(x_con), Some(y_con)) => {
                                    if y_con.clone() == 0 {
                                        return Err(format!("Instruction {} attempted to divide by 0", inst_id));
                                    }
                                    let sum = x_con.clone() / y_con.clone();
                                    let mut val = sum.clone();
                                    if sum < 0 {
                                        val = val.abs();
                                        inst.borrow_mut().update_inst_ty(InstTy::sub);
                                    } else {
                                        inst.borrow_mut().update_inst_ty(InstTy::add);
                                    }
                                    inst.borrow_mut().update_x_val(Value::new(ValTy::con(0)));
                                    inst.borrow_mut().update_y_val(Value::new(ValTy::con(val)));

                                    // Add instruction to value_sub_map
                                    value_sub_map.insert(inst_id, sum);

                                    // After updating the x and y values, remove use of x_op then y_op
                                    temp_manager.borrow_mut_inst(&x_inst_id).borrow_mut().remove_use(&inst_id);
                                    temp_manager.borrow_mut_inst(&y_inst_id).borrow_mut().remove_use(&inst_id);
                                },
                                _ => {},
                            }
                        },
                        // All remaining cases
                        _ => {},
                    }
                },
                InstTy::cmp => {
                    // TODO : This.
                    // Compare the values, if it is solvable at compile time this will remove
                    // unnecessary paths that are not achievable.
                    let mut x_value = 0;
                    let mut y_value = 0;

                    match inst.borrow().get_val_ty() {
                        (Some(ValTy::con(x_val)), Some(ValTy::con(y_val))) => {
                            x_value = x_val;
                            y_value = y_val;
                        },
                        (Some(ValTy::con(x_val)), Some(ValTy::op(y_op))) => {
                            let y_inst_id = y_op.borrow().get_inst_num();
                            if let Some(y_val) = value_sub_map.get(&y_inst_id) {
                                y_value = y_val.clone();
                            } else {
                                println!("Continued on Cmp: {}", inst_id);
                                continue
                            }

                            // TODO : I am Here.
                        },
                        (Some(ValTy::op(x_op)), Some(ValTy::con(y_val))) => {

                        },
                        (Some(ValTy::op(x_op)), Some(ValTy::op(y_op))) => {

                        },
                        _ => {
                            println!("Continued on Cmp: {}", inst_id);
                            continue
                        },
                    }

                    println!("Fell through on Cmp: {}", inst_id);
                    let comp_type = temp_manager.borrow_mut_inst(&inst_id)
                        .borrow().active_uses().last()
                        .expect("All comparisons should be used at least once by the immediately following branch instruction")
                        .clone();
                    let branch_id = comp_type.borrow().inst_num();
                    let branch_type = comp_type.borrow().inst_type();
                    match branch_type.clone() {
                        InstTy::bne => {

                        },
                        InstTy::beq => {  },
                        InstTy::ble => {  },
                        InstTy::blt => {  },
                        InstTy::bge => {  },
                        InstTy::bgt => {  },
                        _ => {
                            return Err(
                                format!("Comparison should not be reference by any type other than branch. Incorrect reference by {:?}",
                                               branch_type))
                        },
                    }
                },
                _ => {
                    // Nothing to do here, these are all the unaffected instructions
                },
            }
        }
    }


    Ok(())
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