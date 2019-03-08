use super::{Graph, IRGraphManager, InstTy, Node, Op, TempValManager, ValTy, Value};
use lib::Optimizer::operator_dominator::OpDomHandler;

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use petgraph::algo::dominators::simple_fast;
use petgraph::algo::dominators::Dominators;
use petgraph::prelude::NodeIndex;
use petgraph::{Directed, Incoming, Outgoing};
use core::borrow::Borrow;

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
                },
                InstTy::load => {
                    if let ValTy::op(load_inst) = inst.borrow()
                        .borrow()
                        .clone_y_val()
                        .unwrap()
                        .get_value()
                        .clone() {
                        // Compare the operation being loaded. If they
                        // are the same, then this load can be deactivated.

                    }
                },
                InstTy::store => {

                },
                _ => {
                    // Do nothing.
                }
            }
        }
    }
}
