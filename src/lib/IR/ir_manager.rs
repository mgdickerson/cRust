use lib::IR::basic_block::BlockTracker;
use lib::IR::ir::{Value,Op,InstTy,InstTracker};
use std::collections::HashMap;
use super::Graph;

pub struct IRManager {
    bt: BlockTracker,
    it: InstTracker,
    var_manager: VariableManager,
    op_dom_handle: OpDomHandler,
}

impl IRManager {
    pub fn new() -> Self {
        IRManager { bt: BlockTracker::new(),
                    it: InstTracker::new(),
                    var_manager: VariableManager::new(),
                    op_dom_handle: OpDomHandler::new()
        }
    }

    pub fn build_op(&self, inst_type: InstTy) -> Op {
        Op::build_op(self.get_inst_num(), self.get_block_num(), inst_type)
    }

    pub fn build_op_x(&self, x_val: Value, inst_type: InstTy) -> Op {
        Op::build_op_x(x_val,self.get_inst_num(),self.get_block_num(),inst_type)
    }

    pub fn build_op_x_y(&self, x_val: Value, y_val: Value, inst_type: InstTy) -> Op {
        Op::build_op_x_y(x_val,
                y_val,
                self.get_inst_num(),
                self.get_block_num(),
                inst_type)
    }

    pub fn build_op_y(&self, y_val: Value, inst_type: InstTy) -> Op {
        Op::build_op_y(y_val, self.get_inst_num(), self.get_block_num(), inst_type)
    }

    pub fn build_spec_op(&self, special_val: Vec<Box<Value>>, inst_type: InstTy) -> Op {
        Op::build_spec_op(special_val,self.get_inst_num(),self.get_block_num(),inst_type)
    }

    pub fn inc_inst_tracker(&mut self) {
        self.it.increment();
    }

    pub fn inc_block_tracker(&mut self) {
        self.bt.increment();
    }

    pub fn get_inst_num(&self) -> usize {
        self.it.get()
    }

    pub fn get_block_num(&self) -> usize {
        self.bt.get()
    }

    pub fn get_var_manager_mut_ref(&mut self) -> &mut VariableManager {
        &mut self.var_manager
    }

    pub fn get_op_dom_manager_mut_ref(&mut self) -> &mut OpDomHandler {
        &mut self.op_dom_handle
    }
}

#[derive(Debug)]
pub struct VariableManager {
    var_manager: HashMap<String, UniqueVariable>,
    var_counter: HashMap<String, usize>,
}

impl VariableManager {
    pub fn new() -> Self {
        VariableManager { var_manager: HashMap::new(), var_counter: HashMap::new() }
    }

    pub fn make_unique_variable(&mut self, ident: String, def: usize) -> &UniqueVariable {
        match self.var_counter.get_mut(&ident) {
            Some(ref mut count) => {
                let current_count = count.clone();
                **count += 1;
                let uniq = UniqueVariable::new(ident.clone(),current_count,def);
                let uniq_key = uniq.get_ident();
                self.var_manager.insert(uniq_key.clone(), uniq);

                return self.var_manager.get(&uniq_key).unwrap();
            }
            None => {
                // variable not found in list, throw error
                panic!("Error: variable ({}) not found within list of variables.");
            }
        }
    }

    pub fn get_unique_variable(&mut self, ident: String, use_site: usize) -> &UniqueVariable {
        match self.var_manager.get_mut(&ident) {
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
            panic!("Variable {} already used", var);
        }
    }

    pub fn is_valid_variable(&self, var: String) -> bool {
        self.var_counter.contains_key(&var)
    }
}

#[derive(Debug)]
pub struct UniqueVariable {
    unique_ident: String,
    def: usize,
    used: Option<Vec<usize>>,
}

impl UniqueVariable {
    pub fn new(ident: String, count: usize, def: usize) -> Self {
        let unique_ident = String::from("%") + &ident + "_" + &count.to_string();
        UniqueVariable { unique_ident, def, used: None }
    }

    pub fn get_ident(&self) -> String {
        self.unique_ident.clone()
    }

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

pub struct OpDomHandler {
    op_manager: HashMap<String, OpGraph>,
}

impl OpDomHandler {
    pub fn new() -> Self {
        OpDomHandler { op_manager: HashMap::new() }
    }

    pub fn get_op_graph(&mut self, op_type: String) -> Option<&mut OpGraph> {
        self.op_manager.get_mut(&op_type)
    }
}

pub struct OpGraph {
    op_graph: Graph<Op,i32>,
    parent_node: Option<petgraph::graph::NodeIndex<u32>>,
}

impl OpGraph {
    pub fn new() -> Self {
        OpGraph { op_graph: Graph::new(), parent_node: None }
    }

    // TODO : I think this structure will have to change to include the desired "Parent Node"
    pub fn add_op(&mut self, child_op: Op, is_sibling: bool) -> petgraph::graph::NodeIndex<u32> {
        let child_node = self.op_graph.add_node(child_op);

        match self.parent_node {
            Some(p_node) => {
                self.op_graph.add_edge(p_node, child_node, 1);
            },
            None => {
                // No need to add edge, this is the first node.
            }
        }

        if is_sibling {
            return child_node;
        }

        self.parent_node = Some(child_node.clone());
        child_node

    }

    pub fn add_child_op(&mut self, parent_node: petgraph::graph::NodeIndex<u32>, child_op: Op) -> petgraph::graph::NodeIndex<u32> {
        let child_node = self.op_graph.add_node(child_op);
        self.op_graph.add_edge(parent_node,child_node.clone(), 1);

        return child_node;
    }

    pub fn get_graph(&self) -> &Graph<Op, i32> {
        &self.op_graph
    }
}