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
    func_params: Vec<String>,
    param_loaded: HashMap<String,bool>,
    local_variables: Vec<String>,
    affected_globals: Vec<String>,
    global_loaded: HashMap<String,bool>,
}

impl UniqueFunction {
    pub fn new(func_name: String) -> Self {
        UniqueFunction { func_name,
            recovery_point: None,
            func_params: Vec::new(),
            param_loaded: HashMap::new(),
            local_variables: Vec::new(),
            affected_globals: Vec::new(),
            global_loaded: HashMap::new(),
        }
    }

    pub fn get_name(&self) -> String {
        self.func_name.clone()
    }

    pub fn add_param(&mut self, func_param: &String) {
        self.func_params.push(func_param.clone());
        self.param_loaded.insert(func_param.clone(), false);
    }

    pub fn check_params(&self, func_param: &String) -> bool {
        self.func_params.contains(func_param)
    }

    pub fn is_param_loaded(&self, func_param: &String) -> bool {
        self.param_loaded.get(func_param).expect("Param should be added before it is loaded.").clone()
    }

    pub fn set_param_loaded(&mut self, func_param: &String) {
        self.param_loaded.insert(func_param.clone(), true);
    }

    pub fn list_func_param(&self) -> Vec<String> {
        self.func_params.clone()
    }

    pub fn add_local(&mut self, local_var: &String) {
        self.local_variables.push(local_var.clone());
    }

    pub fn check_local(&self, local_var: &String) -> bool {
        self.local_variables.contains(local_var)
    }

    pub fn list_local_vars(&self) -> Vec<String> {
        self.local_variables.clone()
    }

    pub fn add_global(&mut self, global_base: &String) {
        self.affected_globals.push(global_base.clone());
        self.global_loaded.insert(global_base.clone(), false);
    }

    pub fn check_global(&self, global_base: &String) -> bool {
        self.affected_globals.contains(global_base)
    }

    pub fn is_global_loaded(&self, func_param: &String) -> bool {
        self.global_loaded.get(func_param).expect("Param should be added before it is loaded.").clone()
    }

    pub fn set_global_loaded(&mut self, func_param: &String) {
        self.global_loaded.insert(func_param.clone(), true);
    }

    pub fn list_affected_globals(&self) -> Vec<String> {
        self.affected_globals.clone()
    }

    pub fn get_load_checkpoint(&self) -> (HashMap<String,bool>,HashMap<String,bool>) {
        (self.param_loaded.clone(), self.global_loaded.clone())
    }

    pub fn recover_load_checkpoint(&mut self, recovery_point: (HashMap<String,bool>,HashMap<String,bool>)) {
        let (param_rec,global_rec) = recovery_point;
        self.param_loaded = param_rec;
        self.global_loaded = global_rec;
    }

    pub fn add_checkpoint(&mut self, checkpoint: HashMap<String, Vec<UniqueVariable>>) {
        self.recovery_point = Some(checkpoint);
    }

    pub fn recover_checkpoint(&self) -> HashMap<String, Vec<UniqueVariable>> {
        self.recovery_point.clone().expect("Should have a recovery point before requesting one.")
    }
}