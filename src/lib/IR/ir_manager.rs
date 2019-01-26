use lib::IR::ir::{Value,ValTy,Op,InstTy};
use std::collections::HashMap;

use lib::Graph::graph_manager::GraphManager;
use lib::Graph::node::{Node,NodeId,NodeData,NodeType};

use super::Graph;
use petgraph::graph::NodeIndex;

/// Rough Draft of IR_Manager Rewrite

pub struct IRGraphManager {
    // Tracker for BlockId, which should match NodeId
    bt: BlockTracker,

    // Tacker for Instruction Id,
    // could also contain the OpDomHandler.
    // Combining the two would allow assignment
    // and possibly assign temp variables for outputs.
    it: InstTracker,
    op_dom_handler: OpDomHandler,

    // User made Variable Tracker
    var_manager: VariableManager,

    // Manages all things graph related.
    graph_manager: GraphManager,
}

// TODO : Add Function for IRGraphManager that either straight adds an Op to the list or Checks (branches not checked, but most others are)

pub struct InstTracker {
    inst_number: usize,
}

impl InstTracker {
    pub fn new() -> InstTracker {
        InstTracker { inst_number: 0 }
    }

    pub fn increment(&mut self) {
        self.inst_number += 1;
    }

    pub fn get(&self) -> usize {
        self.inst_number.clone()
    }
}

#[derive(Clone)]
pub struct BlockTracker {
    block_number: usize,
}

impl BlockTracker {
    pub fn new() -> BlockTracker {
        BlockTracker { block_number: 0 }
    }

    pub fn increment(&mut self) {
        self.block_number += 1;
    }

    pub fn get(&self) -> usize {
        self.block_number.clone()
    }
}

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

    pub fn build_op(&mut self, inst_type: InstTy) -> Op {
        self.inc_inst_tracker();
        Op::build_op(self.get_inst_num(), self.get_block_num(), inst_type)
    }

    pub fn build_op_x(&mut self, x_val: Value, inst_type: InstTy) -> Op {
        self.inc_inst_tracker();
        Op::build_op_x(x_val,self.get_inst_num(),self.get_block_num(),inst_type)
    }

    pub fn build_op_x_y(&mut self, x_val: Value, y_val: Value, inst_type: InstTy) -> Op {
        self.inc_inst_tracker();
        Op::build_op_x_y(x_val,
                y_val,
                self.get_inst_num(),
                self.get_block_num(),
                inst_type)
    }

    pub fn build_op_y(&mut self, y_val: Value, inst_type: InstTy) -> Op {
        self.inc_inst_tracker();
        Op::build_op_y(y_val, self.get_inst_num(), self.get_block_num(), inst_type)
    }

    pub fn build_spec_op(&mut self, special_val: Vec<Box<Value>>, inst_type: InstTy) -> Op {
        self.inc_inst_tracker();
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

    pub fn get_var_manager(self) -> VariableManager { self.var_manager }

    pub fn add_variable(&mut self, ident: String, value: Value) -> &UniqueVariable {
        self.var_manager.add_variable(ident.clone());
        self.make_unique_variable(ident, value)
    }

    pub fn make_unique_variable(&mut self, ident: String, value: Value) -> &UniqueVariable {
        self.var_manager.make_unique_variable(ident, value, self.bt.get(), self.it.get())
    }

    pub fn get_unique_variable(&mut self, ident: String) -> &UniqueVariable {
        self.var_manager.get_unique_variable(ident, self.it.get())
    }

    pub fn get_var_by_block(&self, block_number: usize) -> Option<Vec<String>> {
        self.var_manager.get_variables_by_block(block_number)
    }

    pub fn make_var_table(&self, var: Option<Vec<String>>) -> Option<HashMap<String, UniqueVariable>> {
        self.var_manager.make_var_table(var)
    }

    pub fn get_op_dom_manager_mut_ref(&mut self) -> &mut OpDomHandler {
        &mut self.op_dom_handle
    }
}

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
        self.unique_ident == other.unique_ident
    }
}

pub struct OpDomHandler {
    op_manager: HashMap<InstTy, OpGraph>,
}

impl OpDomHandler {
    pub fn new() -> Self {
        OpDomHandler { op_manager: HashMap::new() }
    }

    pub fn get_op_graph(&mut self, op_type: InstTy) -> Option<&mut OpGraph> {
        self.op_manager.get_mut(&op_type)
    }

    // True means new one was added, should be added to instruction list
    // False means it was found in search, do not add instruction just use value
    pub fn search_or_add_inst(&mut self, new_op: Op) -> (bool, Op) {
        let contains_key = self.op_manager.contains_key(new_op.inst_type());

        if !contains_key {
            let key = new_op.inst_type().clone();
            let op_head = OpNode::new_head_node(new_op.clone());

            self.op_manager.insert(key, OpGraph::new(op_head));

            return (true, new_op);
        }

        let (is_new, op_node) = self.op_manager.get_mut(new_op.inst_type())
            .expect("Key is present, should have graph.")
            .search_or_add(new_op);

        (is_new, op_node.get_op().clone())
    }
}




pub struct OpGraph {
    op_graph: Graph<OpNode,i32>,
    head_node: NodeIndex<u32>,
    tail_node: NodeIndex,
}

impl OpGraph {
    pub fn new(head_node: OpNode) -> Self {
        let mut op_graph = Graph::new();
        let head_node = op_graph.add_node(head_node);
        let tail_node = head_node.clone();
        OpGraph { op_graph, head_node, tail_node }
    }

    pub fn clone_tail_index(&self) -> NodeIndex {
        self.tail_node.clone()
    }

    pub fn revert_tail_index(&mut self, restore_index: NodeIndex) {
        self.tail_node = restore_index;
    }

    // True means one was added, should be added to instruction list
    // False means it was found in search, do not add instruction just use value
    pub fn search_or_add(&mut self, new_op: Op) -> (bool, OpNode) {
        let op_tail = self.op_graph.node_weight(self.tail_node).expect("Tail index should have node weight.").clone();

        // check op_tail
        if op_tail.get_op().clone() == new_op {
            return (false, op_tail);
        }

        // Search through Op chain to find matching Op
        while let Some(op_node) = op_tail.get_parent() {
            if op_node.get_op().clone() == new_op.clone() {
                return (false, op_node.clone());
            }
        }

        // No Op found, add this Op to Op-Chain and return
        let new_tail = OpNode::add_op_node(new_op, op_tail);
        self.tail_node = self.add_child_op(new_tail.clone());

        (true, new_tail)
    }

    pub fn add_child_op(&mut self, child_op: OpNode) -> NodeIndex<u32> {
        let child_node = self.op_graph.add_node(child_op);
        self.op_graph.add_edge(self.tail_node,child_node.clone(), 1);

        return child_node;
    }

    pub fn get_graph(&self) -> &Graph<OpNode, i32> {
        &self.op_graph
    }
}


///
/// General outline of OpGraph
///
///     Graph<Op>
///
///     Op {
///         DominatingParent: Option<Box?<Op>>
///     }
///
///     pub fn get_dom_op(&self) -> &Op {
///         match self.DominatingParent {
///             Some(parent) => { &parent },
///             None => // Do Something Here,
///         }
///     }
///
///     pub fn search_and_replace(&self) -> Op {
///         // Find parent op that matches, or add current Op
///     }
///
///     pub fn add_to_graph(&self, Op1, Op2)
///


#[derive(Clone)]
pub struct OpNode {
    op: Op,
    parent_node: Option<Box<OpNode>>,
}

impl OpNode {
    pub fn new_head_node(op_head: Op) -> Self {
        OpNode { op: op_head, parent_node: None }
    }

    pub fn add_op_node(op: Op, parent_op: OpNode) -> Self {
        OpNode { op, parent_node: Some(Box::new(parent_op)) }
    }

    pub fn get_op(&self) -> &Op {
        &self.op
    }

    pub fn clone_op(&self) -> Op {
        self.op.clone()
    }

    pub fn get_parent(&self) -> Option<OpNode> {
        match self.parent_node.clone() {
            Some(p_node) => {
                Some(*p_node)
            },
            None => None,
        }
    }
}