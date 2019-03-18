use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct AddressManager {
    g_reg_manager: HashMap<String, UniqueAddress>,
    func_reg_manager: HashMap<String, HashMap<String, UniqueAddress>>,
}

impl AddressManager {
    pub fn new() -> Self {
        AddressManager {
            g_reg_manager: HashMap::new(),
            func_reg_manager: HashMap::new(),
        }
    }

    pub fn get_global_reg(&self) -> UniqueAddress {
        UniqueAddress::new(String::from("globalReg"), AddressType::g_reg, 4)
    }

    pub fn get_stack_pointer(&self) -> UniqueAddress {
        UniqueAddress::new(String::from("SP"), AddressType::sp, 4)
    }

    pub fn get_frame_pointer(&self) -> UniqueAddress {
        UniqueAddress::new(String::from("FP"), AddressType::fp, 4)
    }

    pub fn get_addr_assignment(&mut self, addr_name: &String, addr_type: AddressType, size: usize, func_name: Option<String>) -> UniqueAddress {
        let uniq_addr = UniqueAddress::new(addr_name.clone(), addr_type.clone(), size);

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

#[derive(Debug, Clone)]
pub struct UniqueAddress {
    base_ident: String,
    addr_type: AddressType,
    register_value: usize,
}

impl UniqueAddress {
    pub fn new(ident: String, addr_type: AddressType, reg_val: usize) -> Self {
        UniqueAddress {
            base_ident: ident,
            addr_type,
            register_value: reg_val,
        }
    }

    pub fn to_string(&self) -> String {
        let t_str = String::from("&") + &self.base_ident.clone();
        t_str
    }
}

impl PartialEq for UniqueAddress {
    fn eq(&self, other: &UniqueAddress) -> bool {
        self.base_ident == other.base_ident
    }
}

#[derive(PartialEq,Clone,Debug)]
pub enum AddressType {
    g_reg,
    sp,
    fp,
    global_var,
    local_var,
    spill_var,
}