use super::{Graph,GraphManager,Value,ValTy,InstTy, Node, TempValManager, Op};
use super::IRGraphManager;

use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;

use petgraph::prelude::NodeIndex;
use petgraph::{Outgoing,Incoming, Directed};
use lib::Lexer::token::TokenType::Var;

use petgraph::algo::dominators::Dominators;
use petgraph::algo::dominators::simple_fast;
use petgraph::algo::has_path_connecting;

use lib::Utility::display;

pub fn eval_program_constants(irgm: &mut IRGraphManager, temp_manager: &mut TempValManager) -> Result<(), String> {
    println!("Program Eval is being called.");

    // Get mutable reference to the graph manager
    let mut graph_manager = irgm.graph_manager();
    let mut walkable_graph = graph_manager.get_ref_graph().clone();
    let mut inst_graph = graph_manager.get_ref_graph().clone();

    // Get traversal order from temp_manager
    let traversal_order = temp_manager.clone_visit_order();

    // For removing instructions, the plan is to use the number of uses in temp_manager.
    // For functions that are not vital to control flow or operation, once the
    // number of uses drops to 0, it will be marked no longer in use and then
    // cleaned up by a cleaner function.

    let mut value_sub_map : HashMap<usize, i32> = HashMap::new();
    let mut instruction_replacement_map : HashMap<usize, Value> = HashMap::new();
    let mut removed_nodes: Vec<NodeIndex> = Vec::new();

    let mut needs_evaluation = true;
    let mut rounds = 0;

    while needs_evaluation == true {
        needs_evaluation = false;
        for node in traversal_order.iter() {
            if removed_nodes.contains(node) {
                continue
            }
            for inst in inst_graph.node_weight_mut(node.clone()).unwrap().get_mut_data_ref().get_inst_list_ref().iter() {
                let inst_id = inst.borrow().get_inst_num();

                if !temp_manager.borrow_mut_inst(&inst_id).borrow().is_active() {
                    // This instruction is no longer active, skip it.
                    println!("Test this gets called.");
                    continue;
                }

                let inst_ty = inst.borrow().inst_type().clone();
                match generic_type_eval(inst_ty,
                                  inst,
                                  node,
                                  graph_manager,
                                  temp_manager,
                                  & walkable_graph,
                                  &mut value_sub_map,
                                  &mut removed_nodes) {
                    Ok(graph_altered) => {
                        if graph_altered {
                            needs_evaluation = true;
                        }
                    },
                    Err(e) => {
                        return Err(e);
                    }
                }
            }
        }

        //println!("Inactive instructions after single round: ");
        for inst in temp_manager.get_inactive_list() {
            println!("Inactive Instruction: {:?}", inst);
        }

        println!("Finished round {} of eval.", rounds);
        rounds += 1;

        // test purpose
        /*if rounds > 2 {
            needs_evaluation = false;
        }*/
        //println!("Needs evaluation: {}", needs_evaluation);
    }



    Ok(())
}

fn generic_type_eval(inst_ty: InstTy,
                     inst: & Rc<RefCell<Op>>,
                     current_node: & NodeIndex,
                     graph_manager: &mut GraphManager,
                     temp_manager: &mut TempValManager,
                     walkable_graph: & Graph<Node, String, Directed, u32>,
                     value_sub_map: &mut HashMap<usize, i32>,
                     removed_nodes: &mut Vec<NodeIndex>) -> Result<bool, String> {
    if inst_ty == InstTy::cmp {
        return cmp_eval(inst_ty, inst, current_node, graph_manager, temp_manager, walkable_graph, value_sub_map, removed_nodes);
    }

    if inst_ty == InstTy::phi {
        return phi_handler(inst_ty, inst, current_node, graph_manager, temp_manager, walkable_graph, value_sub_map, removed_nodes);
    }

    let inst_id = inst.borrow().get_inst_num();
    let sum;
    match inst.borrow().get_val_ty() {
        (Some(ValTy::con(x_val)),Some(ValTy::con(y_val))) => {
            match inst_ty {
                InstTy::add => {
                    sum = x_val + y_val;
                },
                InstTy::sub => {
                    sum = x_val - y_val;
                },
                InstTy::mul => {
                    sum = x_val * y_val;
                },
                InstTy::div => {
                    if y_val == 0 {
                        return Err(format!("Instruction {} attempted to divide by 0", inst_id));
                    }

                    sum = x_val / y_val;
                },
                _ => {
                    return Ok(false)
                },
            }
        },
        // x_val is const, y_val is an Op
        (Some(ValTy::con(x_val)), Some(ValTy::op(y_op))) => {
            let y_inst_id = y_op.borrow().get_inst_num();

            if let Some(y_val) = value_sub_map.clone().get(&y_inst_id) {
                match inst_ty {
                    InstTy::add => {
                        sum = x_val + y_val.clone();
                    },
                    InstTy::sub => {
                        sum = x_val - y_val.clone();
                    },
                    InstTy::mul => {
                        sum = x_val * y_val.clone();
                    },
                    InstTy::div => {
                        if y_val.clone() == 0 {
                            return Err(format!("Instruction {} attempted to divide by 0", inst_id));
                        }

                        sum = x_val / y_val.clone();
                    },
                    _ => {
                        return Ok(false)
                    },
                }

                // After updating the x and y values, remove use of y_op
                temp_manager.borrow_mut_inst(&y_inst_id).borrow_mut().remove_use(&inst_id);
            } else { return Ok(false) }
        },
        // x_val is an Op, y_val is const
        (Some(ValTy::op(x_op)), Some(ValTy::con(y_val))) => {
            let x_inst_id = x_op.borrow().get_inst_num();

            if let Some(x_val) = value_sub_map.clone().get(&x_inst_id) {
                match inst_ty {
                    InstTy::add => {
                        sum = x_val.clone() + y_val;
                    },
                    InstTy::sub => {
                        sum = x_val.clone() - y_val;
                    },
                    InstTy::mul => {
                        sum = x_val.clone() * y_val;
                    },
                    InstTy::div => {
                        if y_val == 0 {
                            return Err(format!("Instruction {} attempted to divide by 0", inst_id));
                        }

                        sum = x_val.clone() / y_val;
                    },
                    _ => {
                        return Ok(false)
                    },
                }

                // After updating the x and y values, remove use of y_op
                temp_manager.borrow_mut_inst(&x_inst_id).borrow_mut().remove_use(&inst_id);
            } else { return Ok(false) }
        },
        // x_val is an Op, y_val is an Op
        (Some(ValTy::op(x_op)), Some(ValTy::op(y_op))) => {
            let x_inst_id = x_op.borrow().get_inst_num();
            let y_inst_id = y_op.borrow().get_inst_num();

            // Check to see if both have been evaluated
            if let (Some(x_val),Some(y_val)) = (value_sub_map.clone().get(&x_inst_id), value_sub_map.clone().get(&y_inst_id)) {
                match inst_ty {
                    InstTy::add => {
                        sum = x_val.clone() + y_val.clone();
                    },
                    InstTy::sub => {
                        sum = x_val.clone() - y_val.clone();
                    },
                    InstTy::mul => {
                        sum = x_val.clone() * y_val.clone();
                    },
                    InstTy::div => {
                        if y_val.clone() == 0 {
                            return Err(format!("Instruction {} attempted to divide by 0", inst_id));
                        }

                        sum = x_val.clone() / y_val.clone();
                    },
                    _ => {
                        return Ok(false)
                    },
                }

                // After updating the x and y values, remove use of x_op then y_op
                temp_manager.borrow_mut_inst(&x_inst_id).borrow_mut().remove_use(&inst_id);
                temp_manager.borrow_mut_inst(&y_inst_id).borrow_mut().remove_use(&inst_id);
            } else { return Ok(false) }
        },
        // All remaining cases
        _ => {
            return Ok(false)
        },
    }

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

    Ok(false)
}

fn cmp_eval(inst_ty: InstTy,
            inst: & Rc<RefCell<Op>>,
            current_node: & NodeIndex,
            graph_manager: &mut GraphManager,
            temp_manager: &mut TempValManager,
            walkable_graph: & Graph<Node, String, Directed, u32>,
            value_sub_map: &mut HashMap<usize, i32>,
            removed_nodes: &mut Vec<NodeIndex>) -> Result<bool, String> {
    // Compare the values, if it is solvable at compile time this will remove
    // unnecessary paths that are not achievable.
    let inst_id = inst.borrow().get_inst_num();
    let traversal_order = temp_manager.clone_visit_order();
    let node = current_node.clone();

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

                // This value has been solved for, and the comp will be solved as well.
                // Remove use of y_op.
                temp_manager.borrow_mut_inst(&y_inst_id).borrow_mut().remove_use(&inst_id);
            } else {
                //println!("Continued on Cmp: {}", inst_id);
                //println!("Value map at point of continue: {:?}", value_sub_map);
                return Ok(false)
            }

            x_value = x_val;
        },
        (Some(ValTy::op(x_op)), Some(ValTy::con(y_val))) => {
            let x_inst_id = x_op.borrow().get_inst_num();
            if let Some(x_val) = value_sub_map.get(&x_inst_id) {
                x_value = x_val.clone();

                // This value has been solved for, and the comp will be solved as well.
                // Remove use of y_op.
                temp_manager.borrow_mut_inst(&x_inst_id).borrow_mut().remove_use(&inst_id);
            } else {
                //println!("Continued on Cmp: {}", inst_id);
                //println!("Value map at point of continue: {:?}", value_sub_map);

                return Ok(false)
            }

            y_value = y_val;
        },
        (Some(ValTy::op(x_op)), Some(ValTy::op(y_op))) => {
            let x_inst_id = x_op.borrow().get_inst_num();
            let y_inst_id = y_op.borrow().get_inst_num();
            if let (Some(x_val),Some(y_val)) = (value_sub_map.get(&x_inst_id), value_sub_map.get(&y_inst_id)) {
                x_value = x_val.clone();
                y_value = y_val.clone();

                temp_manager.borrow_mut_inst(&x_inst_id).borrow_mut().remove_use(&inst_id);
                temp_manager.borrow_mut_inst(&y_inst_id).borrow_mut().remove_use(&inst_id);
            } else {
                //println!("Continued on Cmp: {}", inst_id);
                //println!("Value map at point of continue: {:?}", value_sub_map);

                return Ok(false)
            }
        },
        _ => {
            //println!("Continued on Cmp: {}", inst_id);
            //println!("Value map at point of continue: {:?}", value_sub_map);

            return Ok(false)
        },
    }

    //println!("Fell through on Cmp: {}", inst_id);
    let mut comp_inst = temp_manager.borrow_mut_inst(&inst_id)
        .borrow().active_uses().last()
        .expect("All comparisons should be used at least once by the immediately following branch instruction")
        .clone();
    let branch_id = comp_inst.borrow().inst_num();
    let branch_type = comp_inst.borrow().inst_type();
    let y_val = comp_inst.borrow().y_val();

    // Deactivate both the comparison instruction AND the branch instruction.
    comp_inst.borrow_mut().deactivate_instruction();
    temp_manager.borrow_mut_inst(&inst_id).borrow_mut().deactivate_instruction();

    // The branch_id is the branch that will be taken if branch type is true.
    if let ValTy::node_id(branch_id) = y_val.unwrap().clone_value() {
        // This variable will keep track of which path to eliminate.
        let mut non_branch_id = branch_id.clone();

        // Walk the two child nodes and get the alternate branch.
        let mut neighbor_walker = walkable_graph.neighbors_directed(node.clone(), Outgoing).detach();
        while let Some(possible_id) = neighbor_walker.next_node(&walkable_graph) {
            if branch_id != possible_id {
                non_branch_id = possible_id;
            }
        }

        let mut eliminate_branch = branch_id;
        match branch_type.clone() {
            InstTy::bne => {
                if x_value != y_value {
                    eliminate_branch = non_branch_id;
                }
            },
            InstTy::beq => {
                if x_value == y_value {
                    eliminate_branch = non_branch_id;
                }
            },
            InstTy::ble => {
                if x_value <= y_value {
                    eliminate_branch = non_branch_id;
                }
            },
            InstTy::blt => {
                if x_value < y_value {
                    eliminate_branch = non_branch_id;
                }
            },
            InstTy::bge => {
                if x_value >= y_value {
                    eliminate_branch = non_branch_id;
                }
            },
            InstTy::bgt => {
                if x_value > y_value {
                    eliminate_branch = non_branch_id;
                }
            },
            _ => {
                panic!("Should not be any type other than direct comparison branches.");
            },
        }

        mark_dead_nodes(graph_manager.get_mut_ref_graph(),
                        node.clone(),
                        eliminate_branch,
                        & traversal_order,
                        removed_nodes,
                        temp_manager);
    }

    //println!("Marking for another round of eval.");
    Ok(true)
}

fn mark_dead_nodes(mut_graph: &mut Graph<Node, String, Directed, u32>,
                   starting_node: NodeIndex,
                   eliminate_node: NodeIndex,
                   traversal_order: & Vec<NodeIndex>,
                   node_removal: &mut Vec<NodeIndex>,
                   temp_manager: &mut TempValManager ) {
    // Visit tracker
    let mut visited : Vec<NodeIndex> = Vec::new();

    let initial_edge = mut_graph.find_edge(starting_node, eliminate_node);
    //println!("Finding Edge between {:?} and {:?} -> Edge : {:?}", starting_node, eliminate_node, initial_edge);
    //println!("Checking if this is the error!");
    mut_graph.remove_edge(initial_edge.unwrap());
    let main_node = traversal_order[0].clone();

    let walkable_graph = mut_graph.clone();

    //println!("Which node is labeled Main Node? {:?}", main_node);

    for node_index in traversal_order.iter() {
        if !has_path_connecting(&walkable_graph, main_node, node_index.clone(), None) {
            if !node_removal.contains(node_index) {
                //println!("Removing node {:?} from graph.", node_index);
                node_removal.push(node_index.clone());
            }

            for inst in mut_graph.node_weight_mut(node_index.clone()).unwrap().get_mut_data_ref().get_inst_list_ref() {
                let inst_num = inst.borrow().get_inst_num();
                temp_manager.borrow_mut_inst(&inst_num).borrow_mut().deactivate_instruction();
            }
        }
    }
}

fn phi_handler(inst_ty: InstTy,
               inst: & Rc<RefCell<Op>>,
               current_node: & NodeIndex,
               graph_manager: &mut GraphManager,
               temp_manager: &mut TempValManager,
               walkable_graph: & Graph<Node, String, Directed, u32>,
               value_sub_map: &mut HashMap<usize, i32>,
               removed_nodes: &mut Vec<NodeIndex>) -> Result<bool, String> {
    let inst_id = inst.borrow().get_inst_num();
    let active_values = temp_manager.check_active_values(&inst_id);
    let traversal_order = temp_manager.clone_visit_order();
    println!("Attemping to resolve Phi: {:?}", inst);
    match active_values {
        (true, true) => {
            return Ok(false);
        },
        (true, false) => {
            let active_x_val = temp_manager.borrow_inst(&inst_id).borrow().x_val().unwrap();

            let active_uses = temp_manager.borrow_mut_inst(&inst_id)
                .borrow().active_uses()
                .iter()
                .map(|temp_val| {
                    temp_val.borrow().inst_val()
            }).collect::<Vec<Rc<RefCell<Op>>>>();
            println!("Active x uses: {:?}", active_uses);
            for op in active_uses {
                op.borrow_mut().op_cleanup(inst_id.clone(), active_x_val.clone());
                //op.borrow_mut().var_cleanup(old_phi_value.clone(), active_x_val.clone());
            }
            //temp_manager.update_inst_uses(&inst_id, active_x_val.clone().unwrap());
            /*for active_use in temp_manager.borrow_inst(&inst_id).borrow().active_uses() {
                let inst_val = active_use.borrow().inst_val();
                inst_val.borrow_mut().var_cleanup(Value::new(ValTy::op(Rc::clone(inst))), active_x_val.clone().unwrap());
            }
            println!("Changing phi use values, then marking phi inactive.");*/
            inst.borrow_mut().deactivate();
            //temp_manager.borrow_mut_inst(&inst_id).borrow_mut().deactivate_instruction();
        },
        (false, true) => {
            let active_y_val = Value::new(temp_manager.borrow_inst(&inst_id).borrow().y_val().unwrap().clone_value());

            let active_uses = temp_manager.borrow_mut_inst(&inst_id)
                .borrow().active_uses()
                .iter()
                .map(|temp_val| {
                    temp_val.borrow().inst_val()
                }).collect::<Vec<Rc<RefCell<Op>>>>();
            println!("Active y uses: {:?}", active_uses);
            for op in active_uses {
                op.borrow_mut().op_cleanup(inst_id.clone(), active_y_val.clone());
                //op.borrow_mut().var_cleanup(old_phi_value.clone(), active_y_val.clone());
            }
            //temp_manager.update_inst_uses(&inst_id, active_y_val.unwrap());
            /*for active_use in temp_manager.borrow_inst(&inst_id).borrow().active_uses() {
                let inst_val = active_use.borrow().inst_val();
                inst_val.borrow_mut().var_cleanup(Value::new(ValTy::op(Rc::clone(inst))), active_y_val.clone().unwrap());
            }
            println!("Changing phi use values, then marking phi inactive.");*/
            inst.borrow_mut().deactivate();
            //temp_manager.borrow_mut_inst(&inst_id).borrow_mut().deactivate_instruction();
        },
        (false, false) => {
            // This should never happen and should be an error...
            println!("Panic on Phi instruction: {}", inst_id);
            //panic!("Should error out here, as these passes should never completely eliminate a Phi.");
        },
    }

    Ok(false)
}