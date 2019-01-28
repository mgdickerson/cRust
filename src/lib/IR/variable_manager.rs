use lib::IR::ir::Value;
use std::collections::HashMap;

#[derive(Debug)]
pub struct VariableManager {
    var_manager: HashMap<String, Vec<UniqueVariable>>,
    var_counter: HashMap<String, usize>,
    block_var: HashMap<usize, Vec<String>>,
}

impl VariableManager {
    pub fn new() -> Self {
        VariableManager { var_manager: HashMap::new(), var_counter: HashMap::new(), block_var: HashMap::new() }
    }

    pub fn get_var_map(self) -> HashMap<String, Vec<UniqueVariable>> {
        self.var_manager
    }

    pub fn get_variables_by_block(&self, block_number: usize) -> Option<Vec<String>> {
        match self.block_var.get(&block_number) {
            Some(vec) => {
                Some(vec.clone())
            }
            None => { None }
        }
    }

    pub fn make_var_table(&self, vars: Option<Vec<String>>) -> Option<HashMap<String, UniqueVariable>> {
        match vars {
            Some(vec) => {
                Some(vec.iter().map(|var| {
                    let uniq_var = self.var_manager
                        .get(var)
                        .expect("Expected Variable in var_manager.")
                        .last()
                        .expect("Expected Latest variable in Vec<Unique Variables>")
                        .clone();
                    (var.clone(), uniq_var)
                }).collect::<HashMap<_,_>>())
            },
            None => {
                None
            },
        }
    }

    pub fn make_unique_variable(&mut self, ident: String, value: Value, def_block: usize, def: usize) -> &UniqueVariable {
        let contains_block = self.block_var.contains_key(&def_block);
        if !contains_block {
            self.block_var.insert(def_block.clone(), Vec::new());
        }

        match self.var_counter.get_mut(&ident) {
            Some(ref mut count) => {
                let current_count = count.clone();
                **count += 1;
                let uniq = UniqueVariable::new(ident.clone(),value,current_count,def_block,def);
                let key = ident;
                self.var_manager.get_mut(&key).expect("Expected established key, found none.").push(uniq);

                let has_var = self.block_var.get_mut(&def_block).expect("Should already be added.").contains(&key);
                if !has_var {
                    self.block_var.get_mut(&def_block).unwrap().push(key.clone());
                }

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