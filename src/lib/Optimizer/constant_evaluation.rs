use super::{Graph,GraphManager,Value,ValTy,InstTy, Node, TempValManager, Op};
use super::IRGraphManager;

use lib::Graph::node::NodeType;

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

// TODO : on while const evaluation, evaluate if the loop will ever even be taken by comparing the right side value of the phi in the cmp inst.
pub fn eval_program_constants(irgm: &mut IRGraphManager, temp_manager: &mut TempValManager, graph_visitor: &Vec<NodeIndex>) -> Result<(), String> {
    //println!("Program Eval is being called.");

    // Get mutable reference to the graph manager
    let mut graph_manager = irgm.graph_manager();
    let mut walkable_graph = graph_manager.get_ref_graph().clone();
    let mut inst_graph = graph_manager.get_ref_graph().clone();

    // Get traversal order from temp_manager
    let traversal_order = graph_visitor.clone();

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
                    //println!("Test this gets called.");
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
                                  &mut removed_nodes,
                                  &graph_visitor) {
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



        println!("Finished round {} of eval.", rounds);
        rounds += 1;
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
                     removed_nodes: &mut Vec<NodeIndex>,
                     graph_visitor: & Vec<NodeIndex> ) -> Result<bool, String> {
    if inst_ty == InstTy::cmp {
        return cmp_eval(inst_ty, inst, current_node, graph_manager, temp_manager, walkable_graph, value_sub_map, removed_nodes, graph_visitor);
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
            removed_nodes: &mut Vec<NodeIndex>,
            graph_visitor: & Vec<NodeIndex> ) -> Result<bool, String> {
    // Compare the values, if it is solvable at compile time this will remove
    // unnecessary paths that are not achievable.
    let inst_id = inst.borrow().get_inst_num();
    let traversal_order = graph_visitor.clone();
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

        let mut children = Vec::new();
        //println!("Children of node: {:?}", node.clone());
        for child_id in graph_manager.get_ref_graph().neighbors(node.clone()) {
            children.push(child_id);
            //println!("Child node: {:?}", child_id);
        }

        let right_path = children[0].clone();
        let left_path = children[1].clone();

        // Walk the two child nodes and get the alternate branch.
        /*let mut neighbor_walker = walkable_graph.neighbors_directed(node.clone(), Outgoing).detach();
        while let Some(possible_id) = neighbor_walker.next_node(&walkable_graph) {
            println!("{:?}", possible_id);
            if branch_id != possible_id {
                non_branch_id = possible_id;
            }
        }*/

        let mut eliminate_branch = right_path;
        match branch_type.clone() {
            InstTy::bne => {
                //println!("{} != {}", x_value, y_value);
                if x_value != y_value {
                    //println!("{} != {} is true!", x_value, y_value);
                    eliminate_branch = left_path;
                }
            },
            InstTy::beq => {
                //println!("{} == {}", x_value, y_value);
                if x_value == y_value {
                    //println!("{} == {}is true!", x_value, y_value);
                    eliminate_branch = left_path;
                }
            },
            InstTy::ble => {
                //println!("{} <= {}", x_value, y_value);
                if x_value <= y_value {
                    //println!("{} <= {} is true!", x_value, y_value);
                    eliminate_branch = left_path;
                }
            },
            InstTy::blt => {
                //println!("{} < {}", x_value, y_value);
                if x_value < y_value {
                    //println!("{} < {} is true!", x_value, y_value);
                    eliminate_branch = left_path;
                }
            },
            InstTy::bge => {
                //println!("{} >= {}", x_value, y_value);
                if x_value >= y_value {
                    //println!("{} >= {} is true!", x_value, y_value);
                    eliminate_branch = left_path;
                }
            },
            InstTy::bgt => {
                //println!("{} > {}", x_value, y_value);
                if x_value > y_value {
                    //println!("{} > {} is true!", x_value, y_value);
                    eliminate_branch = left_path;
                }
            },
            _ => {
                panic!("Should not be any type other than direct comparison branches.");
            },
        }

        mark_dead_nodes(graph_manager,
                        node.clone(),
                        eliminate_branch,
                        & traversal_order,
                        removed_nodes,
                        temp_manager);
    }

    Ok(true)
}

fn mark_dead_nodes(graph_manager: &mut GraphManager,
                   starting_node: NodeIndex,
                   eliminate_node: NodeIndex,
                   traversal_order: & Vec<NodeIndex>,
                   node_removal: &mut Vec<NodeIndex>,
                   temp_manager: &mut TempValManager ) {
    // Visit tracker
    let mut visited : Vec<NodeIndex> = Vec::new();

    let initial_edge = graph_manager.get_mut_ref_graph().find_edge(starting_node, eliminate_node);
    //println!("Finding Edge between {:?} and {:?} -> Edge : {:?}", starting_node, eliminate_node, initial_edge);

    // Remove the edge between block to be removed and the starting block
    graph_manager.get_mut_ref_graph().remove_edge(initial_edge.unwrap());

    // Make new traversal
    let new_traversal_order = graph_manager.graph_visitor(eliminate_node.clone());

    // Main node from original program
    let main_node = traversal_order[0].clone();

    // Get mut graph and a clone for a walkable graph.
    let walkable_graph = graph_manager.get_mut_ref_graph().clone();

    let mut previous_node = new_traversal_order[0].clone();
    for node_index in new_traversal_order.iter() {
        if !has_path_connecting(&walkable_graph, main_node, node_index.clone(), None) {
            if !node_removal.contains(node_index) {
                node_removal.push(node_index.clone());
                graph_manager.add_temp_dominance_edge(node_index.clone(), previous_node, String::from("blue"));

                for inst in graph_manager.get_mut_ref_graph().node_weight_mut(node_index.clone()).unwrap().get_mut_data_ref().get_inst_list_ref() {
                    let inst_id = inst.borrow().get_inst_num();
                    temp_manager.borrow_mut_inst(&inst_id).borrow_mut().deactivate_instruction();
                }
            }

            previous_node = node_index.clone();
        } else {
            // This is the first node reached that DOES have a connecting path, thus it is likely the phi node from initial removal
            let inst_list = graph_manager.get_mut_ref_graph().node_weight_mut(node_index.clone()).unwrap().get_mut_data_ref().get_inst_list_ref().clone();
            for inst in inst_list {
                match inst.borrow().inst_type().clone() {
                    InstTy::phi => {

                    },
                    _ => {
                        continue
                    },
                }
                phi_handler(&inst, &previous_node, node_index, graph_manager, temp_manager);
            }
            break;
        }
    }
}

fn phi_handler(inst: & Rc<RefCell<Op>>,
               parent_node: & NodeIndex,
               current_node: & NodeIndex,
               graph_manager: &mut GraphManager,
               temp_manager: &mut TempValManager) -> Result<bool, String> {
    let inst_id = inst.borrow().get_inst_num();
    //println!("Attemping to resolve Phi: {:?}", inst);
    let active_uses = temp_manager.borrow_mut_inst(&inst_id)
        .borrow().active_uses()
        .iter()
        .map(|temp_val| {
            temp_val.borrow().inst_val()
        }).collect::<Vec<Rc<RefCell<Op>>>>();
    //println!("Active uses: {:?}", active_uses);

    // TODO : When deactivating Phi, traverse up each operand and remove use.

    let node_type = graph_manager.get_ref_graph().node_weight(current_node.clone()).unwrap().get_node_type();

    match node_type {
        NodeType::loop_header => {
            // If this case is reached, it means the path followed was through a
            // loop, thus the left value is no longer "valid" and thus should be
            // replaced with the value on the right.
            let active_y_val = Value::new(temp_manager.borrow_inst(&inst_id).borrow().y_val().unwrap().clone_value());
            let mut y_inst_id : usize = 0;
            if let ValTy::op(y_op) = temp_manager.borrow_inst(&inst_id).borrow().y_val().unwrap().get_value() {
                y_inst_id = y_op.borrow().get_inst_num();
            }

            let active_uses = temp_manager.borrow_mut_inst(&inst_id)
                .borrow().active_uses()
                .iter()
                .map(|temp_val| {
                    temp_val.borrow().inst_val()
                }).collect::<Vec<Rc<RefCell<Op>>>>();
            for op in active_uses {
                // First clean up the old Phi value at instruction site
                op.borrow_mut().op_cleanup(inst_id.clone(), active_y_val.clone());

                // Get instruction id
                let op_id = op.borrow().get_inst_num();
                // Get inst value ref to add to y_inst temp
                let op_temp = temp_manager.borrow_inst(&op_id).clone();

                // Add new use to value used to replace.
                let temp_val = temp_manager.borrow_mut_inst(&y_inst_id);
                temp_val.borrow_mut().add_use(op_temp);
            }
            temp_manager.borrow_mut_inst(&inst_id).borrow_mut().deactivate_instruction();
            temp_manager.clean_instruction_uses(&inst_id);
        },
        NodeType::phi_node => {
            // On Phi nodes, it is important to track which path entered the node.
            // If it was the right path, it will be [0] of the parents, left will
            // be the [1].
            //println!("Parents of node: {:?}", current_node.clone());
            let mut inc_dir = 2;
            for (direction, child_id) in graph_manager.get_ref_graph().neighbors_directed(current_node.clone(), Incoming).enumerate() {
                //println!("Parent node: {:?}", child_id);
                if child_id == parent_node.clone() {
                    inc_dir = direction;
                }
            }

            match inc_dir {
                0 => {
                    // The incoming direction is from the right.
                    // Thus, the right value is no longer valid.
                    let active_x_val = temp_manager.borrow_inst(&inst_id).borrow().x_val().unwrap();
                    let mut x_inst_id : usize = 0;
                    if let ValTy::op(x_op) = temp_manager.borrow_inst(&inst_id).borrow().x_val().unwrap().get_value() {
                        x_inst_id = x_op.borrow().get_inst_num();
                    }

                    let active_uses = temp_manager.borrow_mut_inst(&inst_id)
                        .borrow().active_uses()
                        .iter()
                        .map(|temp_val| {
                            temp_val.borrow().inst_val()
                        }).collect::<Vec<Rc<RefCell<Op>>>>();
                    for op in active_uses {
                        // First clean up the old Phi value at instruction site
                        op.borrow_mut().op_cleanup(inst_id.clone(), active_x_val.clone());

                        // Get instruction id
                        let op_id = op.borrow().get_inst_num();
                        // Get inst value ref to add to y_inst temp
                        let op_temp = temp_manager.borrow_inst(&op_id).clone();

                        // Add new use to value used to replace.
                        let temp_val = temp_manager.borrow_mut_inst(&x_inst_id);
                        temp_val.borrow_mut().add_use(op_temp);
                    }
                    temp_manager.borrow_mut_inst(&inst_id).borrow_mut().deactivate_instruction();
                    temp_manager.clean_instruction_uses(&inst_id);
                },
                1 => {
                    // The incoming direction is from the left.
                    // Thus, the left value is no longer valid, just like with the while loops.
                    let active_y_val = Value::new(temp_manager.borrow_inst(&inst_id).borrow().y_val().unwrap().clone_value());
                    let mut y_inst_id : usize = 0;
                    if let ValTy::op(y_op) = temp_manager.borrow_inst(&inst_id).borrow().y_val().unwrap().get_value() {
                        y_inst_id = y_op.borrow().get_inst_num();
                    }

                    let active_uses = temp_manager.borrow_mut_inst(&inst_id)
                        .borrow().active_uses()
                        .iter()
                        .map(|temp_val| {
                            temp_val.borrow().inst_val()
                        }).collect::<Vec<Rc<RefCell<Op>>>>();
                    for op in active_uses {
                        // First clean up the old Phi value at instruction site
                        op.borrow_mut().op_cleanup(inst_id.clone(), active_y_val.clone());

                        // Get instruction id
                        let op_id = op.borrow().get_inst_num();
                        // Get inst value ref to add to y_inst temp
                        let op_temp = temp_manager.borrow_inst(&op_id).clone();

                        // Add new use to value used to replace.
                        let temp_val = temp_manager.borrow_mut_inst(&y_inst_id);
                        temp_val.borrow_mut().add_use(op_temp);
                    }
                    temp_manager.borrow_mut_inst(&inst_id).borrow_mut().deactivate_instruction();
                    temp_manager.clean_instruction_uses(&inst_id);
                },
                Error => {
                    // Assume this is a case in which the else branch was empty, thus the right side is invalid.
                    let active_x_val = temp_manager.borrow_inst(&inst_id).borrow().x_val().unwrap();
                    let mut x_inst_id : usize = 0;
                    if let ValTy::op(x_op) = temp_manager.borrow_inst(&inst_id).borrow().x_val().unwrap().get_value() {
                        x_inst_id = x_op.borrow().get_inst_num();
                    }

                    let active_uses = temp_manager.borrow_mut_inst(&inst_id)
                        .borrow().active_uses()
                        .iter()
                        .map(|temp_val| {
                            temp_val.borrow().inst_val()
                        }).collect::<Vec<Rc<RefCell<Op>>>>();
                    for op in active_uses {
                        // First clean up the old Phi value at instruction site
                        op.borrow_mut().op_cleanup(inst_id.clone(), active_x_val.clone());

                        // Get instruction id
                        let op_id = op.borrow().get_inst_num();
                        // Get inst value ref to add to y_inst temp
                        let op_temp = temp_manager.borrow_inst(&op_id).clone();

                        // Add new use to value used to replace.
                        let temp_val = temp_manager.borrow_mut_inst(&x_inst_id);
                        temp_val.borrow_mut().add_use(op_temp);
                    }
                    temp_manager.borrow_mut_inst(&inst_id).borrow_mut().deactivate_instruction();
                    temp_manager.clean_instruction_uses(&inst_id);
                },
            }
        },
        _ => {
            // Based on new approach, this should never be reached.
            panic!("Somehow reached a phi statement outside of a loop_header or phi_node.");
        }
    }

    Ok(false)
}