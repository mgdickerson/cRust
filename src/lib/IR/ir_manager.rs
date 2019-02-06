use lib::Parser::AST::number::Number;
use lib::IR::ir::{Value,ValTy,Op,InstTy};
use std::collections::HashMap;

use lib::Graph::graph_manager::GraphManager;
use lib::Graph::node::{Node,NodeId,NodeData,NodeType};

use super::Graph;
use super::variable_manager::{VariableManager, UniqueVariable};
use super::array_manager::{ArrayManager,UniqueArray};
use super::address_manager::{AddressManager,UniqueAddress};
use super::function_manager::{FunctionManager,UniqueFunction};
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

    func_manager: FunctionManager,
    is_func: bool,

    // Manages all things graph related.
    graph_manager: GraphManager,
}

impl IRGraphManager {
    pub fn new() -> Self {
        let graph : Graph<Node, i32> = Graph::new();
        let mut it = InstTracker::new();
        let mut bt = BlockTracker::new();

        let graph_manager = GraphManager::new(graph, &mut it, &mut bt);

        IRGraphManager {
            bt,
            it,
            var_manager: VariableManager::new(),
            array_manager: ArrayManager::new(),
            addr_manager: AddressManager::new(),
            func_manager: FunctionManager::new(),
            is_func: false,
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

        let mut local_var_manager = self.var_manager.clone_self();

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
        }).for_each(|(uniq, uses, phi_inst)| {
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

    pub fn graph_manager(&mut self) -> &mut GraphManager {
        &mut self.graph_manager
    }

    pub fn new_node(&mut self, node_tag: String, node_type: NodeType) -> &NodeIndex {
        let it = &mut self.it;
        let bt = &mut self.bt;
        self.graph_manager.new_node(node_tag, it, bt, node_type)
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

    pub fn variable_manager(&mut self) -> &mut VariableManager {
        &mut self.var_manager
    }

    pub fn get_current_unique(&mut self, ident: String) -> &UniqueVariable {
        let mut block_num = self.get_block_num();
        let mut inst_num = self.get_inst_num() + 1;
        if self.is_func.clone() {
            // Check to see if variable being used is global, and if so has it already been loaded back?
            let is_local = self.var_manager.active_function().check_local(&ident);
            if is_local {
                self.var_manager.get_current_unique(ident,block_num,inst_num)
            } else {
                let global_already_added = self.var_manager.active_function().check_global(&ident);
                if !global_already_added {
                    let var_addr = Value::new(ValTy::adr(self.addr_manager.get_addr_assignment(&ident, 4)));
                    let inst = self.build_op_y(var_addr, InstTy::load);
                    let inst_val = Value::new(ValTy::op(inst.clone()));
                    self.var_manager.make_unique_variable(ident.clone(), inst_val, block_num, inst_num);
                    self.graph_manager.add_instruction(inst);
                    self.inc_block_tracker();
                    self.inc_inst_tracker();
                    block_num = self.get_block_num();
                    inst_num = self.get_inst_num() + 1;
                }
                self.var_manager.get_current_unique(ident,block_num,inst_num)
            }
        } else {
            self.var_manager.get_current_unique(ident,block_num,inst_num)
        }
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

    pub fn array_manager(&mut self) -> &mut ArrayManager {
        &mut self.array_manager
    }

    pub fn build_array_inst(&mut self, uniq_array: UniqueArray, val_vec: Vec<Value>, val_to_assign: Option<Value>) -> Vec<Op> {
        ArrayManager::build_inst(self, uniq_array, val_vec, val_to_assign)
    }

    /// Address Manager ///
    /// These are all just accessors

    pub fn address_manager(&mut self) -> &mut AddressManager {
        &mut self.addr_manager
    }

    /// Function Manager Functions ///

    pub fn function_manager(&mut self) -> &mut FunctionManager {
        &mut self.func_manager
    }

    pub fn new_function(&mut self, func_name: String) {
        self.is_func = true;
        let func = self.func_manager.new_function(&func_name);
        self.array_manager.add_active_function(func.clone());
        self.var_manager.add_active_function(func);
    }

    pub fn end_function(&mut self) {
        self.is_func = false;
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