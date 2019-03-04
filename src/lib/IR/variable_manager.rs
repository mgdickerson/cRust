use lib::IR::function_manager::{FunctionManager, UniqueFunction};
use lib::IR::ir::Op;
use lib::IR::ir::{ValTy, Value};
use std::collections::HashMap;

use super::{Rc, RefCell};

/// General VariableManager Layout
///
/// VarManager {
///     var_man: HashMap<String, UniqueVariable>
///     var_count: HashMap<String, usize>
///     global_vars: Vec<String>
///     active_func: Option<UniqueFunction>
/// }
///
///

#[derive(Debug, Clone)]
pub struct VariableManager {
    var_manager: HashMap<String, Vec<Rc<RefCell<UniqueVariable>>>>,
    current_vars: HashMap<String, Rc<RefCell<UniqueVariable>>>,
    var_counter: HashMap<String, usize>,
    global_vars: Vec<String>,
    active_func: Option<UniqueFunction>,
}

impl VariableManager {
    pub fn new() -> Self {
        VariableManager {
            var_manager: HashMap::new(),
            var_counter: HashMap::new(),
            current_vars: HashMap::new(),
            global_vars: Vec::new(),
            active_func: None,
        }
    }

    pub fn clone_self(&self) -> VariableManager {
        self.clone()
    }

    pub fn get_var_map(self) -> HashMap<String, Vec<Rc<RefCell<UniqueVariable>>>> {
        self.var_manager
    }

    pub fn var_manager(&self) -> &HashMap<String, Vec<Rc<RefCell<UniqueVariable>>>> {
        &self.var_manager
    }

    pub fn get_var_counter(&self) -> HashMap<String, usize> {
        self.var_counter.clone()
    }

    pub fn add_active_function(&mut self, func: UniqueFunction) {
        self.active_func = Some(func);
        match &mut self.active_func {
            Some(uniq_func) => {
                uniq_func.add_checkpoint((self.var_manager.clone(), self.current_vars.clone()));
            }
            None => panic!("Just added, this should not fail."),
        }
    }

    pub fn active_function(&mut self) -> &mut UniqueFunction {
        match &mut self.active_func {
            Some(func) => func,
            None => {
                panic!("Should have active function before referencing it.");
            }
        }
    }

    pub fn get_active_function(&mut self) -> UniqueFunction {
        let uniq_func = self
            .active_func
            .clone()
            .expect("Should have function to recover.");
        match &mut self.active_func {
            Some(func) => {
                let (var_manager, current_vars) = func.recover_checkpoint();
                self.var_manager = var_manager;
                self.current_vars = current_vars;
            }
            None => panic!("Should have failed when cloning."),
        }
        self.active_func = None;
        uniq_func
    }

    pub fn is_global(&self, var: &String) -> bool {
        self.global_vars.contains(var)
    }

    pub fn var_list(&self) -> Vec<String> {
        self.var_manager
            .iter()
            .map(|(key, value)| key.clone())
            .collect::<Vec<String>>()
    }

    pub fn var_checkpoint(&self) -> HashMap<String, Rc<RefCell<UniqueVariable>>> {
        self.current_vars.clone()
    }

    pub fn restore_vars(&mut self, checkpoint: HashMap<String, Rc<RefCell<UniqueVariable>>>) {
        self.current_vars = checkpoint;
    }

    pub fn build_phi_pairs(
        left_set: HashMap<String, Rc<RefCell<UniqueVariable>>>,
        right_set: HashMap<String, Rc<RefCell<UniqueVariable>>>,
    ) -> Vec<(Rc<RefCell<UniqueVariable>>, Rc<RefCell<UniqueVariable>>)> {
        let mut set = left_set
            .iter()
            .filter_map(|(left_ident, left_val)| {
                let right_val = right_set
                    .get(left_ident)
                    .expect("Build Phi Error: Should be present in both.");

                // IS IT YOU?!?!? (it was. they compared val == val, which with aliasing can be the same....)
                //println!("Comparing {} to {} for phi_builder.", var_val.get_ident(), other_val.get_ident());
                if left_val.borrow().get_ident() == right_val.borrow().get_ident() {
                    return None;
                }

                Some((Rc::clone(left_val), Rc::clone(right_val)))
            })
            .collect::<Vec<_>>();
        set.sort_by_key(|(left_key, _right_key)| left_key.borrow().base_ident.clone());

        set
    }

    pub fn loop_correction(&mut self, old_val: Value, new_val: Value) {
        for (ident, uniq_vec) in &mut self.var_manager {
            for uniq in uniq_vec {
                match old_val.clone_value() {
                    ValTy::var(old_var) => {
                        let val_comp = uniq.borrow().get_value().clone_value();
                        match val_comp {
                            ValTy::var(uniq_var) => {
                                if old_var == uniq_var {
                                    //println!("Found a match~! [{:?}] == [{:?}]", old_var, uniq);
                                    uniq.borrow_mut().update_value(new_val.clone());
                                }
                            }
                            _ => {
                                // do nothing,
                            }
                        }
                    }
                    _ => {
                        // Do nothing, like usual.
                    }
                }
            }
        }
    }

    pub fn make_unique_variable(
        &mut self,
        ident: String,
        value: Value,
        def_block: usize,
        def_inst: usize,
    ) -> Rc<RefCell<UniqueVariable>> {
        match self.var_counter.get_mut(&ident) {
            Some(ref mut count) => {
                let current_count = count.clone();
                **count += 1;
                let uniq = Rc::new(RefCell::new(UniqueVariable::new(
                    ident.clone(),
                    value,
                    current_count,
                    def_block,
                    def_inst,
                )));
                let key = ident;
                self.var_manager
                    .get_mut(&key)
                    .expect("Expected established key, found none.")
                    .push(Rc::clone(&uniq));

                // Add/Update to current_vars map
                self.current_vars.insert(key.clone(), Rc::clone(&uniq));

                //println!("Current Key: {}\tCurrent Count: {}", key, current_count);

                return Rc::clone(
                    self.var_manager
                        .get(&key)
                        .unwrap()
                        .last()
                        .expect("There should be a last as one was just inserted."),
                );
            }
            None => {
                // variable not found in list, throw error
                panic!(
                    "Error: variable ({}) not found within list of variables.",
                    ident
                );
            }
        }
    }

    pub fn get_current_unique(&mut self, ident: String) -> Rc<RefCell<UniqueVariable>> {
        let current_uniq = self
            .current_vars
            .get(&ident)
            .expect("Expected variable, found none.");
        Rc::clone(current_uniq)
    }

    pub fn get_latest_unique(&mut self, ident: String) -> Rc<RefCell<UniqueVariable>> {
        let latest_unique = self
            .var_manager
            .get(&ident)
            .expect("Expected variable to be constained, none was found.")
            .last()
            .expect("Should have at least one variable when getting latest.");
        Rc::clone(latest_unique)
    }

    pub fn add_variable(&mut self, var: &String, value: Value, block_num: usize, inst_num: usize) {
        match &mut self.active_func {
            Some(active_func) => {
                self.var_counter.insert(var.clone(), 0);
            }
            None => {
                let var_already_added = self.var_counter.insert(var.clone(), 0);

                if var_already_added != None {
                    panic!("Variable {} already used", var.clone());
                }
            }
        }

        self.var_manager.insert(var.clone(), Vec::new());
        self.make_unique_variable(var.clone(), value, block_num, inst_num);
    }

    pub fn add_global(&mut self, var: &String, value: Value, block_num: usize, inst_num: usize) {
        self.global_vars.push(var.clone());
        self.add_variable(var, value, block_num, inst_num);
    }

    // Make this a general add use?
    pub fn add_var_use(
        &mut self,
        uniq: Rc<RefCell<UniqueVariable>>,
        block_num: usize,
        inst_num: usize,
    ) {
        // This is currently just for use in the Phi construction, thus I want to return
        // the uniq value just before i add the usage at the Phi site.
        uniq.borrow_mut().add_use(block_num, inst_num);
    }

    pub fn is_valid_variable(&self, var: String) -> bool {
        self.var_counter.contains_key(&var)
    }
}

// Giving the value an Rc<RefCell<Box<Value>>> so that it I change its contents, it will be changes properly.
#[derive(Debug, Clone)]
pub struct UniqueVariable {
    unique_ident: String,
    base_ident: String,
    value: Value,
    def_block: usize,
    def_inst: usize,
    used: Option<Vec<(usize, usize)>>,
}

impl UniqueVariable {
    pub fn new(
        ident: String,
        value: Value,
        count: usize,
        def_block: usize,
        def_inst: usize,
    ) -> Self {
        let base_ident = ident.clone();
        let unique_ident = String::from("%") + &ident + "_" + &count.to_string();
        UniqueVariable {
            unique_ident,
            base_ident,
            value,
            def_block,
            def_inst,
            used: None,
        }
    }

    pub fn get_ident(&self) -> String {
        self.unique_ident.clone()
    }

    pub fn get_base_ident(&self) -> String {
        self.base_ident.clone()
    }

    pub fn update_value(&mut self, new_val: Value) {
        self.value = new_val;
    }

    pub fn get_value(&self) -> Value {
        self.value.clone()
    }

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

    pub fn get_uses(&self) -> Option<Vec<(usize, usize)>> {
        self.used.clone()
    }

    pub fn get_block(&self) -> usize {
        self.def_block.clone()
    }

    pub fn add_use(&mut self, block_num: usize, inst_num: usize) {
        match &mut self.used {
            Some(uses_vec) => {
                if uses_vec.contains(&(block_num, inst_num)) {
                    return;
                }
                uses_vec.push((block_num, inst_num));
                return;
            }
            None => {
                // pass through
            }
        }

        // this will only hit if use vector is not already present
        self.used = Some(Vec::new());
        match &mut self.used {
            Some(some) => some.push((block_num, inst_num)),
            None => panic!("Unreachable Error."),
        }
    }

    pub fn remove_use(&mut self, block_num: usize, inst_num: usize) {
        match &mut self.used {
            Some(uses_vec) => {
                let uses_clone = uses_vec.clone();
                for (iter, location) in uses_clone.iter().enumerate() {
                    let (block_match, inst_match) = location;
                    if *block_match == block_num && *inst_match == inst_num {
                        uses_vec.remove(iter);
                    }
                }
            }
            None => {
                panic!("Attempted to remove use location but there are no uses.");
            }
        }
    }
}

impl PartialEq for UniqueVariable {
    // I think i have found the issue once more. I think this should be a string comparison, not a value comparison.
    fn eq(&self, other: &UniqueVariable) -> bool {
        self.unique_ident == other.unique_ident
    }
}
