use lib::IR::variable_manager::{VariableManager, UniqueVariable};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct FunctionManager {
    func_manager: HashMap<String,UniqueFunction>,
}

impl FunctionManager {
    pub fn new() -> Self {
        FunctionManager { func_manager: HashMap::new() }
    }

    pub fn new_function(&mut self, func_name: &String) -> UniqueFunction {
        UniqueFunction::new(func_name.clone())
    }

    pub fn get_mut_function(&mut self, func_name: & String) -> &mut UniqueFunction {
        self.func_manager.get_mut(func_name).expect("Attempted to get non-existent function.")
    }

    pub fn add_func_to_manager(&mut self, func: UniqueFunction) {
        self.func_manager.insert(func.get_name(), func);

    }
}

#[derive(Debug, Clone)]
pub struct UniqueFunction {
    func_name: String,
    recovery_point: Option<HashMap<String, Vec<UniqueVariable>>>,
    local_variables: Vec<String>,
    affected_globals: Vec<String>,
}

impl UniqueFunction {
    pub fn new(func_name: String) -> Self {
        UniqueFunction { func_name, recovery_point: None, local_variables: Vec::new(), affected_globals: Vec::new() }
    }

    pub fn get_name(&self) -> String {
        self.func_name.clone()
    }

    pub fn add_local(&mut self, local_var: &String) {
        self.local_variables.push(local_var.clone());
    }

    pub fn check_local(&self, local_var: &String) -> bool {
        self.local_variables.contains(local_var)
    }

    pub fn add_global(&mut self, global_base: &String) {
        self.affected_globals.push(global_base.clone());
    }

    pub fn check_global(&self, global_base: &String) -> bool {
        self.affected_globals.contains(global_base)
    }

    pub fn add_checkpoint(&mut self, checkpoint: HashMap<String, Vec<UniqueVariable>>) {
        self.recovery_point = Some(checkpoint);
    }

    pub fn recover_checkpoint(&self) -> HashMap<String, Vec<UniqueVariable>> {
        self.recovery_point.clone().expect("Should have a recovery point before requesting one.")
    }
}