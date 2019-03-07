use super::{IRGraphManager, TempValManager};

pub struct SpillHandler {
    current_spill_counter: usize
}

impl SpillHandler {
    pub fn new() -> Self {
        SpillHandler { current_spill_counter: 0 }
    }

    pub fn spill_value(
        irgm: &mut IRGraphManager,
        temp_manager: &mut TempValManager,
        inst_id: usize
    ) {
        // Grab definition block from temp_manager
        let inst_def_block = temp_manager.borrow_inst(&inst_id).borrow().block_num();
        let inst_node_id = irgm.graph_manager()
            .block_node_map()
            .get(&inst_def_block)
            .unwrap()
            .clone();

        
    }
}