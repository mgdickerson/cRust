use lib::IR::ir::Value;
use std::collections::HashMap;
use lib::IR::ir::{Op,OpPosition};

#[derive(Debug, Clone)]
pub struct VariableManager {
    var_manager: HashMap<String, Vec<UniqueVariable>>,
    var_counter: HashMap<String, usize>,
    current_vars: HashMap<String, UniqueVariable>,
}

impl VariableManager {
    pub fn new() -> Self {
        VariableManager { var_manager: HashMap::new(), var_counter: HashMap::new(), current_vars: HashMap::new() }
    }

    pub fn get_var_map(self) -> HashMap<String, Vec<UniqueVariable>> {
        self.var_manager
    }

    pub fn clone_current_vars(&self) -> HashMap<String, UniqueVariable> {
        self.current_vars.clone()
    }

    pub fn restore_vars(&mut self, checkpoint: HashMap<String, UniqueVariable>) {
        self.current_vars = checkpoint;
    }

    pub fn build_phi_pairs(left_set: HashMap<String, UniqueVariable>, right_set: HashMap<String, UniqueVariable>)
        -> Vec<(UniqueVariable, UniqueVariable)> {
        let mut set = left_set.iter()
            .filter_map(|(var_ident,var_val)| {
                let other_val = right_set
                    .get(var_ident)
                    .expect("Build Phi Error: Should be present in both.")
                    .clone();

                if var_val.clone() == other_val {
                    return None;
                }

                Some((var_val.clone(), other_val))
            }).collect::<Vec<_>>();
        set.sort_by_key(|(left_key, right_key)| {
            left_key.base_ident.clone()
        });

        set
    }

    pub fn make_unique_variable(&mut self, ident: String, value: Value, def_block: usize, def: usize) -> &UniqueVariable {
        match self.var_counter.get_mut(&ident) {
            Some(ref mut count) => {
                let current_count = count.clone();
                **count += 1;
                let uniq = UniqueVariable::new(ident.clone(),value,current_count,def_block,def);
                let key = ident;
                self.var_manager.get_mut(&key).expect("Expected established key, found none.").push(uniq.clone());

                // Add/Update to current_vars map
                self.current_vars.insert(key.clone(), uniq);

                return self.var_manager.get(&key).unwrap().get(current_count).expect("Expected current count to work.");
            }
            None => {
                // variable not found in list, throw error
                panic!("Error: variable ({}) not found within list of variables.");
            }
        }
    }

    pub fn get_unique_variable(&mut self, ident: String, use_site: usize) -> &UniqueVariable {
        match self.var_manager.get_mut(&ident).expect("Expected variable, found none.").last_mut() {
            Some(uniq) => {
                uniq.add_use(use_site);
                uniq
            }
            None => {
                panic!("Error: key {} not found in var_manager", ident);
            }
        }
    }

    pub fn add_variable(&mut self, var: String) {
        let var_already_added = self.var_counter.insert(var.clone(), 0);

        if var_already_added != None {
            panic!("Variable {} already used", var.clone());
        }

        self.var_manager.insert(var, Vec::new());
    }

    pub fn is_valid_variable(&self, var: String) -> bool {
        self.var_counter.contains_key(&var)
    }
}

#[derive(Debug, Clone)]
pub struct UniqueVariable {
    unique_ident: String,
    base_ident: String,
    value: Box<Value>,
    def_block: usize,
    def: usize,
    used: Option<Vec<usize>>,
}

impl UniqueVariable {
    pub fn new(ident: String, value: Value, count: usize, def_block: usize, def: usize) -> Self {
        let base_ident = ident.clone();
        let unique_ident = String::from("%") + &ident + "_" + &count.to_string();
        UniqueVariable { unique_ident, base_ident, value: Box::new(value), def_block, def, used: None }
    }

    pub fn get_ident(&self) -> String {
        self.unique_ident.clone()
    }

    pub fn get_base_ident(&self) -> String { self.base_ident.clone() }

    pub fn get_value(&self) -> Box<Value> { self.value.clone() }

    pub fn value_to_string(&self) -> String {
        self.value.get_value().to_string()
    }

    pub fn get_block(&self) -> usize { self.def_block.clone() }

    pub fn add_use(&mut self, var_use: usize) {
        match &mut self.used {
            Some(uses_vec) => {
                uses_vec.push(var_use);
                return
            },
            None => {
                // pass through
            }
        }

        // this will only hit if use vector is not already present
        self.used = Some(Vec::new());
        match &mut self.used {
            Some(some) => some.push(var_use),
            None => { panic!("Unreachable Error.") },
        }
    }
}

impl PartialEq for UniqueVariable {
    fn eq(&self, other: &UniqueVariable) -> bool {
        self.value == other.value
    }
}