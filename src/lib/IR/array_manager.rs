use std::collections::HashMap;
use lib::Parser::AST::number::Number;
use lib::IR::ir::{Value, ValTy, Op, InstTy};
use lib::IR::ir_manager::IRGraphManager;
use lib::IR::address_manager::UniqueAddress;
use lib::IR::function_manager::{FunctionManager,UniqueFunction};

use super::{Rc,RefCell};

#[derive(Debug, Clone)]
pub struct ArrayManager {
    array_manager: HashMap<String, UniqueArray>,
    array_global: Vec<String>,
    active_func: Option<UniqueFunction>,
}

impl ArrayManager {
    pub fn new() -> Self {
        ArrayManager{ array_manager: HashMap::new(), array_global: Vec::new(), active_func: None }
    }

    pub fn add_active_function(&mut self, func: UniqueFunction) {
        self.active_func = Some(func);
    }

    pub fn is_global(&self, array_ident: &String) -> bool {
        self.array_global.contains(array_ident)
    }

    pub fn add_global(&mut self, array_ident: &String, array_depth: Vec<Number>) {
        self.array_global.push(array_ident.clone());
        self.add_array(array_ident, array_depth);
    }

    pub fn add_array(&mut self, array_ident: &String, array_depth: Vec<Number>) {
        self.array_manager.insert(array_ident.clone(), UniqueArray::new(array_ident.clone(), array_depth, None));
    }

    pub fn assign_addr(&mut self, array_ident: String, uniq_addr: UniqueAddress) {
        self.array_manager.get_mut(&array_ident).expect("array should exist before assigning address.").assign_address(uniq_addr);
    }

    pub fn get_array_ref(&self, array_ident: String) -> &UniqueArray {
        match self.array_manager.get(&array_ident) {
            Some(arr_ident) => {
                arr_ident
            },
            None => panic!("Attempted to get array that is not present."),
        }
    }

    pub fn build_inst(irgm: &mut IRGraphManager, uniq_arr: UniqueArray, val_vec: Vec<Value>, value_to_assign: Option<Value>) -> Value {
        let mut last_offset_inst : Option<Value> = None;

        val_vec.iter()
            .enumerate()
            .for_each(|(adjust, val)| {
                // adjust needs +1 because when storing space for the array, the first element is not counted for adjustment
                // The 4 is for byte size of i32 (standard size for this project)
                let adj_size = 4 * uniq_arr.generate_adjustment(adjust + 1);

                if let ValTy::con(val_con) = val.get_value().clone() {
                    let new_adjust = val_con * adj_size;
                    let new_adjust_value = Value::new(ValTy::con(new_adjust));
                    match last_offset_inst.clone() {
                        Some(last_inst) => {
                            if let ValTy::con(last_con) = last_inst.clone().get_value() {
                                last_offset_inst = Some(Value::new(ValTy::con(last_con + new_adjust)));
                            } else {
                                let offset_inst = irgm.build_op_x_y(last_inst, new_adjust_value, InstTy::add);
                                let latest_offset = irgm.graph_manager().add_instruction(offset_inst);
                                last_offset_inst = Some(latest_offset);
                            }
                        },
                        None => {
                            last_offset_inst = Some(new_adjust_value);
                        }
                    }
                } else {
                    let adjustment =
                        Value::new(
                            ValTy::con(adj_size.clone())
                        );

                    // Generate Offset for Array
                    let mul_inst = irgm.build_op_x_y(val.clone(), adjustment, InstTy::mul);
                    let mul_val = irgm.graph_manager().add_instruction(mul_inst);

                    match last_offset_inst.clone() {
                        Some(last_inst) => {
                            let offset_inst = irgm.build_op_x_y(last_inst, mul_val, InstTy::add);
                            let last_offset_val = irgm.graph_manager().add_instruction(offset_inst);
                            last_offset_inst = Some(last_offset_val);
                        },
                        None => {
                            last_offset_inst = Some(mul_val);
                        },
                    }
                }
        });

        let final_offset;
        if let ValTy::con(final_con) = last_offset_inst.clone().expect("Should be at least one instruction").get_var_base() {
            let add_load = irgm.build_op_x_y(Value::new(ValTy::con(0)), last_offset_inst.unwrap(), InstTy::add);
            final_offset = irgm.graph_manager().add_instruction(add_load);
        } else {
            final_offset = last_offset_inst.expect("Should be at least one instruction.");
        }

        // Find Array home register
        let ref_register;
        if irgm.array_manager().is_global(&uniq_arr.base_ident) {
            ref_register = Value::new(ValTy::adr(irgm.address_manager().get_global_reg()));
        } else {
            ref_register = Value::new(ValTy::adr(irgm.address_manager().get_frame_pointer()));
        }
        let arr_reg = Value::new(ValTy::adr(uniq_arr.clone_addr()));
        let add_inst = irgm.build_op_x_y(ref_register, arr_reg, InstTy::add);
        let array_reg_value = irgm.graph_manager().add_instruction(add_inst);

        // Adda offset to home register
        let adda_inst = irgm.build_op_x_y(final_offset, array_reg_value, InstTy::adda);
        let adda_val = irgm.graph_manager().add_instruction(adda_inst);

        let ret_val;
        // if there is a value to assign, store, otherwise load.
        match &value_to_assign {
            Some(val) => {
                let mut val_to_assign = val.clone();
                if let ValTy::con(val_store) = val.get_value().clone() {
                    let add_inst = irgm.build_op_x_y(Value::new(ValTy::con(0)), val.clone(), InstTy::add);
                    val_to_assign = irgm.graph_manager().add_instruction(add_inst);
                }
                let store_inst = irgm.build_op_x_y(adda_val, val_to_assign, InstTy::store);
                ret_val = irgm.graph_manager().add_instruction(store_inst);
            },
            None => {
                let load_inst = irgm.build_op_y(adda_val, InstTy::load);
                ret_val = irgm.graph_manager().add_instruction(load_inst);
            },
        }

        ret_val
    }
}

// TODO : Will likely have to add storage for value
#[derive(Debug, Clone)]
pub struct UniqueArray {
    base_ident: String,
    array_depth: Vec<i32>,
    uniq_addr: Option<UniqueAddress>,
}

impl UniqueArray {
    pub fn new(arr_ident: String, arr_depth: Vec<Number>, uniq_addr: Option<UniqueAddress>) -> Self {
        let array_depth = arr_depth.iter()
            .map(|num| {
                num.get_value()
        }).collect::<Vec<i32>>();
        UniqueArray { base_ident: arr_ident, array_depth, uniq_addr }
    }

    pub fn assign_address(&mut self, uniq_addr: UniqueAddress) {
        self.uniq_addr = Some(uniq_addr);
    }

    pub fn clone_addr(&self) -> UniqueAddress {
        self.uniq_addr.clone().expect("Address not available to clone")
    }

    pub fn generate_adjustment(&self, adjust: usize) -> i32 {
        let iter = self.array_depth.iter().skip(adjust);
        iter.product()
    }

    pub fn to_string(&self) -> String {
        let t_str = String::from("%") + &self.base_ident;
        t_str
    }

    pub fn get_size(&self) -> usize {
        let size : i32 = self.array_depth.iter().product();
        size as usize
    }
}

impl PartialEq for UniqueArray {
    fn eq(&self, other: &UniqueArray) -> bool {
        self.base_ident == other.base_ident
    }
}