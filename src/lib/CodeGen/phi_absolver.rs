use lib::IR::ir_manager::IRGraphManager;
use std::collections::HashMap;

// Remove phis by adding mov (add R0 (op)) or by ensuring that a phi shares the same register as its two operands.
pub fn remove_phis(irgm: &mut IRGraphManager, register_map: &mut HashMap<usize, usize>) {
    let node_walker = irgm.graph_manager_ref().get_ref_graph().node_indices();

    for node_id in node_walker {
        let inst_list = irgm
            .graph_manager_ref()
            .get_ref_graph()
            .node_weight(node_id)
            .unwrap()
            .get_data_ref()
            .get_inst_list_ref()
            .clone();
    }
}
