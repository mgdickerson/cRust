use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct AddressManager {
    register_manager: HashMap<String, UniqueAddress>,
    current_available_reg: usize,
    base_register: usize,
    global_register: usize,
}

impl AddressManager {
    pub fn new() -> Self {
        AddressManager {
            register_manager: HashMap::new(),
            current_available_reg : 0,
            base_register : 0,
            global_register : 0,
        }
    }

    pub fn get_global_reg(&self) -> UniqueAddress {
        UniqueAddress::new(String::from("globalReg"), self.global_register.clone())
    }

    pub fn get_base_reg(&self) -> UniqueAddress {
        UniqueAddress::new(String::from("baseReg"), self.base_register.clone())
    }

    pub fn get_addr_assignment(&mut self, addr_name: String, size: usize) -> UniqueAddress {
        let current_clone = self.current_available_reg.clone();
        self.current_available_reg += size;

        UniqueAddress::new(addr_name, current_clone)
    }
}

#[derive(Debug, Clone)]
pub struct UniqueAddress {
    base_ident: String,
    register_value: usize,
}

impl UniqueAddress {
    pub fn new(ident: String, reg_val: usize) -> Self {
        UniqueAddress { base_ident: ident, register_value: reg_val }
    }

    pub fn to_string(&self) -> String {
        let t_str = String::from("%") + &self.base_ident.clone();
        t_str
    }
}

impl PartialEq for UniqueAddress {
    fn eq(&self, other: &UniqueAddress) -> bool {
        self.base_ident == other.base_ident
    }
}