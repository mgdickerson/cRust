use super::{IRGraphManager, InstTy, Node, NodeIndex, TempValManager};

/// Dead Code Elimination
pub fn dead_code_elimination(
    irgm: &mut IRGraphManager,
    temp_manager: &mut TempValManager,
    root_node: NodeIndex,
) {
    // Unlike previous passes, this one will traverse in reverse order.
    let mut visitor = irgm.graph_manager().graph_visitor(root_node);
    visitor.reverse();

    // TODO : Figure out how many times this needs to run to be accurate.
    let mut value_deactivated = true;
    let mut test_counter = 0;
    while value_deactivated {
        value_deactivated = false;
        println!("DCE pass: {:?}", test_counter);
        test_counter += 1;

        for node_id in &visitor {
            // Each instruction will also need to be traversed in reverse order as well.
            let mut inst_list = irgm
                .graph_manager()
                .get_mut_ref_graph()
                .node_weight_mut(node_id.clone())
                .unwrap()
                .get_mut_data_ref()
                .get_mut_inst_list_ref()
                .clone();

            inst_list.reverse();

            for inst in inst_list {
                //println!("Checking instruction: {:?}", inst);
                if InstTy::kill == inst.borrow().inst_type().clone() {
                    continue
                }

                let active_uses = temp_manager
                    .borrow_inst(&inst.borrow().get_inst_num())
                    .borrow()
                    .active_uses();

                if active_uses.len() < 1 {
                    if inst.borrow().is_active() {
                        value_deactivated = true;
                        //println!("Removing inactive instruction: {:?}", inst);
                    }
                    // If the number of times it is used is less than 1,
                    // then the instruction is inactive and should be marked.
                    inst.borrow_mut().deactivate();
                }
            }
        }
    }
}
