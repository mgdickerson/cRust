use lib::Parser::AST::number::Number;
use lib::IR::ir::{Value,ValTy,Op,InstTy};
use std::collections::HashMap;

use lib::Graph::graph_manager::GraphManager;
use lib::Graph::node::{Node,NodeId,NodeData,NodeType};

use super::Graph;
use super::variable_manager::{VariableManager, UniqueVariable};
use super::array_manager::{ArrayManager,UniqueArray};
use super::address_manager::{AddressManager,UniqueAddress};
use super::operator_dominator::{OpDomHandler,OpNode,OpGraph};
use petgraph::graph::NodeIndex;
use petgraph::algo::dominators::Dominators;
use petgraph::algo::dominators;

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

    // User made Array tracker
    array_manager: ArrayManager,

    // User made Address Manager (for use with arrays and stack variables)
    addr_manager: AddressManager,

    // Manages all things graph related.
    graph_manager: GraphManager,
}

impl IRGraphManager {
    pub fn new() -> Self {
        let mut graph : Graph<Node, i32> = Graph::new();
        let mut it = InstTracker::new();
        let mut bt = BlockTracker::new();

        let graph_manager = GraphManager::new(graph, &mut it, &mut bt);

        IRGraphManager {
            bt,
            it,
            var_manager: VariableManager::new(),
            array_manager: ArrayManager::new(),
            addr_manager: AddressManager::new(),
            op_dom_handler: OpDomHandler::new(),
            graph_manager,
        }
    }

    /// Op Specific Functions ///

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

    pub fn loop_variable_correction(&mut self, vars: Vec<(UniqueVariable,usize)>) -> Vec<(UniqueVariable,usize,usize)> {
        // Grab current_node ID so that we dont alter any uses before the occurrence of this node.
        let current_node = self.graph_manager.clone_node_index();
        let node_starting_point = self.graph_manager.get_node_id(current_node);
        let mut remove_use_vec = Vec::new();

        let mut local_var_manager = self.var_manager.get_self();

        // Make map of current graph.
        let mut graph_map = self.graph_manager.get_mut_ref_graph()
            .node_weights_mut()
            .map(|node| {
                let block_num = node.get_node_id();
                (block_num, node)
            }).collect::<HashMap<usize,&mut Node>>();

        // Perform iteration and correction
        vars.iter().filter_map(|(uniq, phi_inst)| {
            let uniq_clone = uniq.clone();
            match uniq.get_uses() {
                Some(uses) => Some((uniq_clone, uses, phi_inst)),
                None => None,
            }
        }).for_each(|(uniq,mut uses, phi_inst)| {
            println!("Current Node Id: {}\tPhi Inst: {}", node_starting_point.clone(), phi_inst);
            for (block_num, inst_num) in uses {
                println!("Uniq: {}\tBlock: {}\tInst: {}", uniq.get_ident(), block_num, inst_num);
                if block_num >= node_starting_point {
                    remove_use_vec.push((uniq.clone(),block_num,inst_num));
                    let node = graph_map.get_mut(&block_num).expect("Block number should exist");
                    for inst in node.get_mut_data_ref().get_mut_ref() {
                        if inst.get_inst_num() != phi_inst.clone() {
                            let uniq_base = uniq.get_base_ident();
                            let old_val = Value::new(ValTy::var(uniq.clone()));
                            let new_val = Value::new(ValTy::var(local_var_manager.get_latest_unique(uniq_base,block_num,inst_num).clone()));
                            inst.var_cleanup(old_val,new_val);
                        }
                    }
                }
            }

        });

        //println!("Uses to Remove: {:?}", remove_use_vec.clone());
        self.var_manager = local_var_manager;
        remove_use_vec
    }

    /// Graph Specific Functions ///

    pub fn get_mut_ref_graph(&mut self) -> &mut GraphManager {
        &mut self.graph_manager
    }

    pub fn get_graph(self) -> Graph<Node, i32> { self.graph_manager.get_graph() }

    pub fn new_node(&mut self, node_type: NodeType) -> &NodeIndex {
        let it = &mut self.it;
        let bt = &mut self.bt;
        self.graph_manager.new_node(it, bt, node_type)
    }

    pub fn clone_node_index(&self) -> NodeIndex {
        self.graph_manager.clone_node_index()
    }

    pub fn switch_current_node(&mut self, new_node: NodeIndex) {
        self.graph_manager.switch_current_node_index(new_node);
    }

    pub fn get_node_id(&self, node_index: NodeIndex) -> usize {
        self.graph_manager.get_node_id(node_index)
    }

    pub fn add_edge(&mut self, parent: NodeIndex, child: NodeIndex) {
        self.graph_manager.add_edge(parent, child);
    }

    pub fn try_add_inst(&mut self, inst: Op) -> Op {
        let (new_inst, op) = self.op_dom_handler.search_or_add_inst(inst);

        if new_inst {
            self.add_inst(op.clone());
        }

        op
    }

    pub fn add_inst(&mut self, inst: Op) -> Op {
        self.graph_manager.add_instruction(inst.clone());

        inst
    }

    /// Tracker Specific Functions ///

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
        let current_node = self.graph_manager.clone_node_index();
        self.graph_manager.get_node_id(current_node)
    }

    /// Variable Manager Specific Functions ///

    pub fn get_var_manager_mut_ref(&mut self) -> &mut VariableManager {
        &mut self.var_manager
    }

    pub fn get_var_manager(self) -> VariableManager { self.var_manager }

    pub fn clone_var_manager(&self) -> VariableManager { self.var_manager.clone() }

    pub fn add_variable(&mut self, ident: String, value: Value) -> &UniqueVariable {
        self.var_manager.add_variable(ident.clone());
        self.make_unique_variable(ident, value)
    }

    pub fn make_unique_variable(&mut self, ident: String, value: Value) -> &UniqueVariable {
        let block_num = self.get_block_num();
        self.var_manager.make_unique_variable(ident, value, block_num, self.it.get())
    }

    pub fn get_current_unique(&mut self, ident: String) -> &UniqueVariable {
        let block_num = self.get_block_num();
        self.var_manager.get_current_unique(ident, block_num, self.it.get() + 1)
    }

    pub fn get_latest_unique(&mut self, ident: String) -> &UniqueVariable {
        let block_num = self.get_block_num();
        self.var_manager.get_latest_unique(ident, block_num, self.it.get() + 1)
    }

    pub fn var_checkpoint(&self) -> HashMap<String, UniqueVariable> {
        self.var_manager.clone_current_vars()
    }

    pub fn restore_vars(&mut self, checkpoint: HashMap<String, UniqueVariable>) {
        self.var_manager.restore_vars(checkpoint);
    }

    pub fn remove_uses(&mut self, uses_to_remove: Vec<(UniqueVariable,usize,usize)>) {
        for (uniq, block_num, inst_num) in uses_to_remove {
            let mut uniq_result = self.var_manager.get_mut_uniq_var(uniq);
            match uniq_result {
                Ok(mut_uniq) => {
                    mut_uniq.remove_use(block_num, inst_num);
                },
                Err(e) => panic!(e),
            }
        }
    }

    pub fn insert_phi_inst(&mut self, left_set: HashMap<String, UniqueVariable>, right_set: HashMap<String, UniqueVariable>)
        -> Vec<(UniqueVariable, usize)> {
        let phi_set = VariableManager::build_phi_pairs(left_set, right_set);
        let mut inst_position = 0;
        let mut while_touch_up_vars = Vec::new();

        for (left_var, right_var) in phi_set {
            let block_num = self.get_block_num();
            let inst_num = self.it.get();

            let left_uniq = self.var_manager.add_phi_uniq_use(left_var, block_num, inst_num + 1);
            let right_uniq = self.var_manager.add_phi_uniq_use(right_var, block_num, inst_num + 1);

            let left_val = Value::new(ValTy::var(left_uniq.clone()));
            let right_val = Value::new(ValTy::var(right_uniq.clone()));
            let inst = self.build_op_x_y(left_val, right_val, InstTy::phi);

            // make new unique variable with phi value
            self.var_manager.make_unique_variable(left_uniq.get_base_ident(),
                Value::new(ValTy::op(inst.clone())),
                block_num,
                inst_num + 1);

            //while_touch_up_vars.push(left_uniq.clone());
            while_touch_up_vars.push((right_uniq, inst_num + 1));

            self.graph_manager.insert_instruction(inst_position, inst);
            inst_position += 1;
        }

        while_touch_up_vars
    }

    /// Array Manager Specific Functions ///

    pub fn add_array(&mut self, array_ident: String, array_depth: Vec<Number>) {
        self.array_manager.add_array(array_ident, array_depth);
    }

    pub fn assign_array_address(&mut self, array_ident: String, uniq_addr: UniqueAddress) {
        self.array_manager.assign_addr(array_ident, uniq_addr);
    }

    pub fn get_array_ref(&self, array_ident: String) -> &UniqueArray {
        self.array_manager.get_array_ref(array_ident)
    }

    pub fn build_array_inst(&mut self, uniq_array: UniqueArray, val_vec: Vec<Value>, val_to_assign: Option<Value>) -> Vec<Op> {
        ArrayManager::build_inst(self, uniq_array, val_vec, val_to_assign)
    }

    /// Address Manager ///

    pub fn get_global_addr(&self) -> UniqueAddress {
        self.addr_manager.get_global_reg()
    }

    pub fn get_base_addr(&self) -> UniqueAddress {
        self.addr_manager.get_base_reg()
    }

    pub fn get_addr_assignment(&mut self, addr_name: String, size: usize) -> UniqueAddress {
        self.addr_manager.get_addr_assignment(addr_name, size)
    }

    /// Op Dominator Specific Functions ///

    pub fn get_op_dom_manager_mut_ref(&mut self) -> &mut OpDomHandler {
        &mut self.op_dom_handler
    }

    pub fn get_op_dom(self) -> OpDomHandler {
        self.op_dom_handler
    }

    pub fn get_op_graph(&mut self, op_type: InstTy) -> Option<&mut OpGraph> {
        self.op_dom_handler.get_op_graph(op_type)
    }

    pub fn set_op_recovery_point(&mut self) -> OpDomHandler {
        self.op_dom_handler.set_recovery_point()
    }

    pub fn restore_op(&mut self, op_dom_handler: OpDomHandler) {
        self.op_dom_handler.restore(op_dom_handler);
    }
}

#[derive(Clone)]
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