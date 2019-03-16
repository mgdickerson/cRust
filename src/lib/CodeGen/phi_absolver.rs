use lib::IR::ir_manager::IRGraphManager;
use std::collections::HashMap;
use lib::IR::ir::{ValTy,Value};
use lib::IR::ir::InstTy;
use lib::Graph::node::NodeType;

use petgraph::{Directed, Incoming, Outgoing};
use lib::RegisterAllocator::RegisterAllocation;

// Remove phis by adding mov (add R0 (op)) or by ensuring that a phi shares the same register as its two operands.
pub fn remove_phis(irgm: &mut IRGraphManager, register_map: &mut HashMap<usize, usize>) {
    let node_walker = irgm.graph_manager_ref().get_ref_graph().node_indices();

    for node_id in node_walker.clone() {
        let node_type = irgm
            .graph_manager_ref()
            .get_ref_graph()
            .node_weight(node_id)
            .unwrap()
            .get_node_type();

        // Only phi_nodes and while_loop_header nodes contain phis,
        // speed up the process by skipping the rest.
        if node_type != NodeType::phi_node && node_type != NodeType::while_loop_header {
            //println!("Skipping non-phi or while_loop node: {:?}", node_type);
            continue
        }

        let inst_list = irgm
            .graph_manager_ref()
            .get_ref_graph()
            .node_weight(node_id)
            .unwrap()
            .get_data_ref()
            .get_inst_list_ref()
            .clone();

        for inst in inst_list {
            let inst_type = inst.borrow().inst_type().clone();
            if inst_type == InstTy::phi {
                // First check easy case where phi has the same register as operands
                let phi_id = inst.borrow().get_inst_num();
                let phi_register = register_map.get(&phi_id).unwrap().clone();

                let phi_operands = inst.borrow().get_val_ty();
                if let (Some(ValTy::op(x_op)), Some(ValTy::op(y_op))) = phi_operands.clone() {
                    let x_register = register_map.get(&x_op.borrow().get_inst_num()).unwrap().clone();
                    let y_register = register_map.get(&y_op.borrow().get_inst_num()).unwrap().clone();

                    // If all these match, we can take the easy path and just remove the phi
                    if phi_register == x_register && phi_register == y_register {
                        //println!("All registers in phi match!");
                        inst.borrow_mut().deactivate();
                        continue
                    }
                }

                let mut neighbor_walker = irgm
                    .graph_manager_ref()
                    .get_ref_graph()
                    .neighbors_directed(node_id, Incoming)
                    .detach();

                let mut x_parent_node = node_id.clone();
                let mut y_parent_node = node_id.clone();
                if node_type == NodeType::while_loop_header {
                    while let Some(parent_id) = neighbor_walker.next_node(irgm.graph_manager_ref().get_ref_graph()) {
                        if parent_id.index() < node_id.index() {
                            y_parent_node = parent_id;
                        } else {
                            x_parent_node = parent_id;
                        }
                    }
                } else {
                    let mut parent_vec = Vec::new();
                    while let Some(parent_id) = neighbor_walker.next_node(irgm.graph_manager_ref().get_ref_graph()) {
                        parent_vec.push(parent_id);
                    }
                    x_parent_node = parent_vec.pop().unwrap();
                    y_parent_node = parent_vec.pop().unwrap();
                }

                let r0_value = Value::new(ValTy::reg(RegisterAllocation::allocate_R0()));
                let phi_register_val = Value::new(ValTy::reg(RegisterAllocation::allocate_register(phi_register.clone())));

                // If function reaches this point, it means
                // there will be a need to add instructions
                match phi_operands {
                    (Some(ValTy::op(x_op)), _) => {
                        let x_register = register_map.get(&x_op.borrow().get_inst_num()).unwrap().clone();

                        if phi_register != x_register {
                            // x-register does not match and needs to be added
                            let x_add_op = irgm.build_op_x_y_in_block(
                                phi_register_val.clone(),
                                inst.borrow().clone_x_val().unwrap(),
                                InstTy::mov,
                                x_parent_node.index());

                            irgm.graph_manager().add_instruction_in_node(x_add_op.clone(), &x_parent_node);
                            register_map.insert(x_add_op.get_inst_num(), phi_register.clone());
                        }

                        let y_add_op = irgm.build_op_x_y_in_block(
                            phi_register_val.clone(),
                            inst.borrow().clone_y_val().unwrap(),
                            InstTy::mov,
                            y_parent_node.index());

                        irgm.graph_manager().add_instruction_in_node(y_add_op.clone(), &y_parent_node);
                        register_map.insert(y_add_op.get_inst_num(), phi_register.clone());
                    },
                    (_, Some(ValTy::op(y_op))) => {
                        let y_register = register_map.get(&y_op.borrow().get_inst_num()).unwrap().clone();

                        if phi_register != y_register {
                            // y-register does not match and needs to be added
                            let y_add_op = irgm.build_op_x_y_in_block(
                                phi_register_val.clone(),
                                inst.borrow().clone_y_val().unwrap(),
                                InstTy::mov,
                                y_parent_node.index());

                            irgm.graph_manager().add_instruction_in_node(y_add_op.clone(), &y_parent_node);
                            register_map.insert(y_add_op.get_inst_num(), phi_register.clone());
                        }

                        let x_add_op = irgm.build_op_x_y_in_block(
                            phi_register_val.clone(),
                            inst.borrow().clone_x_val().unwrap(),
                            InstTy::mov,
                            x_parent_node.index());

                        irgm.graph_manager().add_instruction_in_node(x_add_op.clone(), &x_parent_node);
                        register_map.insert(x_add_op.get_inst_num(), phi_register.clone());
                    },
                    _ => {
                        // x-register does not match and needs to be added
                        let x_add_op = irgm.build_op_x_y_in_block(
                            phi_register_val.clone(),
                            inst.borrow().clone_x_val().unwrap(),
                            InstTy::mov,
                            x_parent_node.index());

                        irgm.graph_manager().add_instruction_in_node(x_add_op.clone(), &x_parent_node);
                        register_map.insert(x_add_op.get_inst_num(), phi_register.clone());

                        // y-register does not match and needs to be added
                        let y_add_op = irgm.build_op_x_y_in_block(
                            phi_register_val.clone(),
                            inst.borrow().clone_y_val().unwrap(),
                            InstTy::mov,
                            y_parent_node.index());

                        irgm.graph_manager().add_instruction_in_node(y_add_op.clone(), &y_parent_node);
                        register_map.insert(y_add_op.get_inst_num(), phi_register.clone());
                    },
                }

                inst.borrow_mut().deactivate();
            }
        }
    }

    // After invalidating all Phi instructions and correctly inserting all add instructions, remove phis.
    for node_index in node_walker {
        irgm.graph_manager().remove_inactive_inst(node_index);
    }
}
