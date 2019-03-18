use std::collections::HashMap;
use lib::IR::address_manager::AddressType::g_reg;

#[derive(Debug, Clone)]
pub struct AddressManager {
    g_reg_manager: HashMap<String, UniqueAddress>,
    global_assignments: HashMap<UniqueAddress, i32>,
    func_reg_manager: HashMap<String, HashMap<String, UniqueAddress>>,
    function_assignments: HashMap<String, HashMap<UniqueAddress, i32>>,
}

impl AddressManager {
    pub fn new() -> Self {
        AddressManager {
            g_reg_manager: HashMap::new(),
            global_assignments: HashMap::new(),
            func_reg_manager: HashMap::new(),
            function_assignments: HashMap::new(),
        }
    }

    pub fn get_global_reg(&self) -> UniqueAddress {
        UniqueAddress::new(String::from("globalReg"), AddressType::g_reg, 4, None)
    }

    pub fn get_stack_pointer(&self) -> UniqueAddress {
        UniqueAddress::new(String::from("SP"), AddressType::sp, 4, None)
    }

    pub fn get_frame_pointer(&self) -> UniqueAddress {
        UniqueAddress::new(String::from("FP"), AddressType::fp, 4, None)
    }

    pub fn set_variable_assignments(&mut self) {
        // Set all global assignments
        let mut global_offset = 0;
        for (key, value) in self.g_reg_manager.iter() {
            self.global_assignments.insert(value.clone(), global_offset.clone());
            global_offset += value.get_size() as i32;
        }

        // Set all function local assignments
        for (func_name, func_manager) in self.func_reg_manager.iter() {
            let mut func_offset = 0;
            let mut func_assignments = HashMap::new();
            for (key, value) in func_manager.iter() {
                func_assignments.insert(value.clone(), func_offset.clone());
                func_offset += value.get_size() as i32;
            }

            self.function_assignments.insert(func_name.clone(), func_assignments);
        }
    }

    pub fn get_assignment(&self, uniq_addr: &UniqueAddress) -> i32 {
        let is_global = uniq_addr.is_global();
        if is_global {
            self.global_assignments.get(uniq_addr).unwrap().clone()
        } else {
            let func_name = uniq_addr.get_func_name();
            self.function_assignments.get(&func_name).unwrap().get(uniq_addr).unwrap().clone()
        }
    }

    pub fn get_addr_assignment(&mut self, addr_name: &String, addr_type: AddressType, size: usize, func_name: Option<String>) -> UniqueAddress {
        let uniq_addr = UniqueAddress::new(addr_name.clone(), addr_type.clone(), size, func_name.clone());

        if addr_type == AddressType::global_var {
            if !self.g_reg_manager.contains_key(addr_name) {
                self.g_reg_manager.insert(addr_name.clone(), uniq_addr.clone());
            }
        } else if addr_type == AddressType::local_var {
            match self.func_reg_manager.clone().get(&func_name.clone().unwrap()) {
                Some(func_manager) => {
                    if !func_manager.contains_key(addr_name) {
                        self.func_reg_manager
                            .get_mut(&func_name.clone().unwrap())
                            .unwrap()
                            .insert(addr_name.clone(), uniq_addr.clone());
                    }
                },
                None => {
                    let mut new_hashmap = HashMap::new();
                    new_hashmap.insert(addr_name.clone(), uniq_addr.clone());
                    self.func_reg_manager.insert(func_name.unwrap(), new_hashmap);
                }
            }
        } else if addr_type == AddressType::spill_var {
            match func_name {
                Some(name) => {
                    match self.func_reg_manager.clone().get(&name) {
                        Some(func_manager) => {
                            if !func_manager.contains_key(addr_name) {
                                self.func_reg_manager.get_mut(&name).unwrap().insert(addr_name.clone(), uniq_addr.clone());
                            }
                        },
                        None => {
                            let mut new_hashmap = HashMap::new();
                            new_hashmap.insert(addr_name.clone(), uniq_addr.clone());
                            self.func_reg_manager.insert(name, new_hashmap);
                        }
                    }
                },
                None => {
                    if !self.g_reg_manager.contains_key(addr_name) {
                        self.g_reg_manager.insert(addr_name.clone(), uniq_addr.clone());
                    }
                },
            }
        }

        uniq_addr
    }
}

#[derive(Debug, Clone, Hash, Eq)]
pub struct UniqueAddress {
    base_ident: String,
    addr_type: AddressType,
    value_size: usize,
    func_name: Option<String>,
}

impl UniqueAddress {
    pub fn new(ident: String, addr_type: AddressType, reg_val: usize, func_name: Option<String>) -> Self {
        UniqueAddress {
            base_ident: ident,
            addr_type,
            value_size: reg_val,
            func_name,
        }
    }

    pub fn get_type(&self) -> AddressType {
        self.addr_type.clone()
    }

    pub fn get_size(&self) -> usize {
        self.value_size.clone()
    }

    pub fn to_string(&self) -> String {
        let t_str = String::from("&") + &self.base_ident.clone();
        t_str
    }

    pub fn get_func_name(&self) -> String {
        self.func_name.clone().unwrap().clone()
    }

    pub fn is_global(&self) -> bool {
        match &self.func_name {
            Some(name) => {
                false
            },
            None => true,
        }
    }
}

impl PartialEq for UniqueAddress {
    fn eq(&self, other: &UniqueAddress) -> bool {
        self.base_ident == other.base_ident
    }
}

#[derive(PartialEq,Clone,Debug, Hash, Eq)]
pub enum AddressType {
    g_reg,
    sp,
    fp,
    global_var,
    local_var,
    spill_var,
}