use super::{IRGraphManager, TempValManager};
use lib::IR::ir::{Value, ValTy, InstTy};
use lib::RegisterAllocator::RegisterAllocation;
use std::cell::RefCell;
use std::rc::Rc;

pub struct SpillHandler {
    current_spill_counter: usize
}

impl SpillHandler {
    pub fn new() -> Self {
        SpillHandler { current_spill_counter: 0 }
    }

    pub fn spill_value(
        &mut self,
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

        let spill_string = String::from("spill_val") + &self.current_spill_counter.to_string();
        self.current_spill_counter += 1;

        let uniq_spill_addr = irgm.address_manager()
            .get_addr_assignment(
                &spill_string,
                4
            );

        let spill_addr_value = Value::new(ValTy::adr(uniq_spill_addr.clone()));

        let mut inst_list = irgm.graph_manager()
            .get_ref_graph()
            .node_weight(inst_node_id)
            .unwrap()
            .get_data_ref()
            .get_inst_list_ref()
            .clone();
        for (position, inst) in inst_list.iter().enumerate() {
            if inst.borrow().get_inst_num() == inst_id {
                let add_op = irgm.build_op_x_y_in_block(
                    Value::new(ValTy::reg(RegisterAllocation::allocate_R0())),
                    spill_addr_value.clone(),
                    InstTy::add,
                    inst_def_block.clone()
                );

                let storage_location = irgm.graph_manager()
                    .insert_instruction_in_node(
                        position + 1,
                        add_op,
                        &inst_node_id
                    );

                let store_op = irgm.build_op_x_y_in_block(
                    storage_location,
                    Value::new(ValTy::op(Rc::clone(inst))),
                    InstTy::store,
                    inst_def_block.clone()
                );

                irgm.graph_manager()
                    .insert_instruction_in_node(
                        position + 2,
                        store_op,
                        &inst_node_id
                    );
            }
        }

        for active_use in temp_manager.borrow_inst(&inst_id).borrow().active_uses() {
            let inst_use_node_id = irgm.graph_manager()
                .block_node_map()
                .get(&active_use.borrow().block_num())
                .unwrap()
                .clone();

            let mut inst_list = irgm.graph_manager()
                .get_ref_graph()
                .node_weight(inst_use_node_id)
                .unwrap()
                .get_data_ref()
                .get_inst_list_ref()
                .clone();

            for (position, inst) in inst_list.iter().enumerate() {
                let inst_block = inst.borrow().get_inst_block();
                if inst.borrow().get_inst_num() == active_use.borrow().inst_num() {
                    let add_op = irgm.build_op_x_y_in_block(
                        Value::new(ValTy::reg(RegisterAllocation::allocate_R0())),
                        spill_addr_value.clone(),
                        InstTy::add,
                        inst_block.clone()
                    );

                    let storage_location = irgm.graph_manager()
                        .insert_instruction_in_node(
                            position,
                            add_op,
                            &inst_use_node_id
                        );

                    let load_op = irgm.build_op_y_in_block(
                        storage_location,
                        InstTy::load,
                        inst_block.clone()
                    );

                    let load_value = irgm.graph_manager()
                        .insert_instruction_in_node(
                            position + 1,
                            load_op,
                            &inst_use_node_id
                        );

                    inst.borrow_mut().op_cleanup(inst_id, load_value);
                }
            }
        }
    }
}