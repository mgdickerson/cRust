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

    pub fn make_unique_variable(&mut self, ident: String, value: Value, def_block: usize, def_inst: usize) -> &UniqueVariable {
        match self.var_counter.get_mut(&ident) {
            Some(ref mut count) => {
                let current_count = count.clone();
                **count += 1;
                let uniq = UniqueVariable::new(ident.clone(),value,current_count,def_block,def_inst);
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

    pub fn get_latest_unique(&mut self, ident: String, block_num: usize, inst_num: usize) -> &UniqueVariable {
        match self.var_manager.get_mut(&ident).expect("Expected variable, found none.").last_mut() {
            Some(uniq) => {
                uniq.add_use(block_num, inst_num);
                uniq
            }
            None => {
                panic!("Error: key {} not found in var_manager", ident);
            }
        }
    }

    pub fn get_uniq_variable(&self, uniq_lookup: UniqueVariable) -> UniqueVariable {
        let uniq_vec = self.var_manager.get(&uniq_lookup.get_base_ident()).unwrap();
        for uniq in uniq_vec {
            if uniq_lookup.get_ident() == uniq.get_ident() {
                return uniq.clone()
            }
        }

        // This is basically an error case
        // TODO : Make this a Result return?
        uniq_lookup
    }

    pub fn add_variable(&mut self, var: String) {
        let var_already_added = self.var_counter.insert(var.clone(), 0);

        if var_already_added != None {
            panic!("Variable {} already used", var.clone());
        }

        self.var_manager.insert(var, Vec::new());
    }

    pub fn add_phi_uniq_use(&mut self, uniq: UniqueVariable, block_num: usize, inst_num: usize) -> UniqueVariable {
        // This is currently just for use in the Phi construction, thus I want to return
        // the uniq value just before i add the usage at the Phi site.
        let uniq_copy = self.get_uniq_variable(uniq.clone());

        self.var_manager
            .get_mut(&uniq.get_base_ident())
            .expect("This value should appear in var_manager")
            .iter_mut()
            .for_each(|uniq_var| {
                if uniq_var.get_ident() == uniq.get_ident() {
                    uniq_var.add_use(block_num,inst_num);
                }
            });

        uniq_copy
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
    def_inst: usize,
    used: Option<Vec<(usize,usize)>>,
}

impl UniqueVariable {
    pub fn new(ident: String, value: Value, count: usize, def_block: usize, def_inst: usize) -> Self {
        let base_ident = ident.clone();
        let unique_ident = String::from("%") + &ident + "_" + &count.to_string();
        UniqueVariable { unique_ident, base_ident, value: Box::new(value), def_block, def_inst, used: None }
    }

    pub fn get_ident(&self) -> String {
        self.unique_ident.clone()
    }

    pub fn get_base_ident(&self) -> String { self.base_ident.clone() }

    pub fn get_value(&self) -> Box<Value> { self.value.clone() }

    pub fn value_to_string(&self) -> String {
        self.value.get_value().to_string()
    }

    pub fn get_ident_val(&self) -> String {
        let mut ret_string = self.unique_ident.clone();
        ret_string += "<";
        ret_string += &self.value_to_string();
        ret_string += ">";
        ret_string
    }

    pub fn get_uses(&self) -> Option<Vec<(usize,usize)>> {
        self.used.clone()
    }

    pub fn get_block(&self) -> usize { self.def_block.clone() }

    pub fn add_use(&mut self, block_num: usize, inst_num: usize) {
        match &mut self.used {
            Some(uses_vec) => {
                uses_vec.push((block_num,inst_num));
                return
            },
            None => {
                // pass through
            }
        }

        // this will only hit if use vector is not already present
        self.used = Some(Vec::new());
        match &mut self.used {
            Some(some) => some.push((block_num,inst_num)),
            None => { panic!("Unreachable Error.") },
        }
    }
}

impl PartialEq for UniqueVariable {
    fn eq(&self, other: &UniqueVariable) -> bool {
        self.value == other.value
    }
}