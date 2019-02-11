use std::collections::HashMap;
use lib::Parser::AST::number::Number;
use lib::IR::ir::{Value, ValTy, Op, InstTy};
use lib::IR::ir_manager::IRGraphManager;
use lib::IR::address_manager::UniqueAddress;
use lib::IR::function_manager::{FunctionManager,UniqueFunction};

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

    pub fn build_inst(irgm: &mut IRGraphManager, uniq_arr: UniqueArray, val_vec: Vec<Value>, value_to_assign: Option<Value>) -> Vec<Op> {
        let mut inst_vec = Vec::new();
        let mut last_offset_inst : Option<Value> = None;

        val_vec.iter()
            .enumerate()
            .for_each(|(adjust, val)| {
                // adjust needs +1 because when storing space for the array, the first element is not counted for adjustment
                // The 4 is for byte size of i32 (standard size for this project)
                let adjustment = Value::new(ValTy::con(4 * uniq_arr.generate_adjustment(adjust + 1)));

                // Generate Offset for Array
                let mul_inst = irgm.build_op_x_y(val.clone(), adjustment, InstTy::mul);
                inst_vec.push(mul_inst.clone());

                match last_offset_inst.clone() {
                    Some(last_inst) => {
                        let mul_val = Value::new(ValTy::op(mul_inst));
                        let offset_inst = irgm.build_op_x_y(last_inst, mul_val, InstTy::add);
                        inst_vec.push(offset_inst.clone());
                        last_offset_inst = Some(Value::new(ValTy::op(offset_inst)));
                    },
                    None => {
                        last_offset_inst = Some(Value::new(ValTy::op(mul_inst)));
                    },
                }

        });

        let last_mul_inst = inst_vec.last().expect("Should be at least one instruction.").clone();

        // Find Array home register
        // TODO : I believe this is the spot to change.
        let ref_register;
        if irgm.array_manager().is_global(&uniq_arr.base_ident) {
            ref_register = Value::new(ValTy::adr(irgm.address_manager().get_global_reg()));
        } else {
            ref_register = Value::new(ValTy::adr(irgm.address_manager().get_frame_pointer()));
        }
        let arr_reg = Value::new(ValTy::adr(uniq_arr.clone_addr()));
        let add_inst = irgm.build_op_x_y(ref_register, arr_reg, InstTy::add);
        inst_vec.push(add_inst.clone());

        // Adda offset to home register
        let offset_val = Value::new(ValTy::op(last_mul_inst));
        let arr_reg_val = Value::new(ValTy::op(add_inst));
        let adda_inst = irgm.build_op_x_y(offset_val, arr_reg_val, InstTy::adda);
        inst_vec.push(adda_inst.clone());

        let adda_val = Value::new(ValTy::op(adda_inst));
        // if there is a value to assign, store, otherwise load.
        match &value_to_assign {
            Some(val) => {
                let store_inst = irgm.build_op_x_y(adda_val, val.clone(), InstTy::store);
                inst_vec.push(store_inst.clone());
            },
            None => {
                let load_inst = irgm.build_op_y(adda_val, InstTy::load);
                inst_vec.push(load_inst);
            },
        }

        inst_vec
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