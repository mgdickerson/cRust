use super::{Rc, RefCell};
use lib::Parser::AST::number::Number;
use lib::IR::ir::{InstTy, Op, ValTy, Value};
use std::collections::HashMap;

use lib::Graph::graph_manager::GraphManager;
use lib::Graph::node::{Node, NodeData, NodeId, NodeType};

use super::address_manager::{AddressManager, UniqueAddress};
use super::array_manager::{ArrayManager, UniqueArray};
use super::function_manager::{FunctionManager, UniqueFunction};
use super::variable_manager::{UniqueVariable, VariableManager};
use super::Graph;
use petgraph::algo::dominators;
use petgraph::algo::dominators::Dominators;
use petgraph::graph::NodeIndex;

/// Rough Draft of IR_Manager Rewrite
#[derive(Clone)]
pub struct IRGraphManager {
    // Tracker for BlockId, which should match NodeId
    bt: BlockTracker,

    // Tacker for Instruction Id,
    // could also contain the OpDomHandler.
    // Combining the two would allow assignment
    // and possibly assign temp variables for outputs.
    it: InstTracker,

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
        let graph = Graph::new();
        let mut it = InstTracker::new();
        let mut bt = BlockTracker::new();

        let graph_manager = GraphManager::new(graph, &mut bt);

        IRGraphManager {
            bt,
            it,
            var_manager: VariableManager::new(),
            array_manager: ArrayManager::new(),
            addr_manager: AddressManager::new(),
            func_manager: FunctionManager::new(),
            is_func: false,
            graph_manager,
        }
    }

    pub fn is_func(&self) -> bool {
        self.is_func
    }

    /// Op Specific Functions ///

    pub fn build_op(&mut self, inst_type: InstTy) -> Op {
        self.inc_inst_tracker();
        Op::build_op(
            None,
            None,
            None,
            self.get_block_num(),
            self.get_inst_num(),
            inst_type,
            &mut self.var_manager,
        )
    }

    pub fn build_op_in_block(&mut self, inst_type: InstTy, block_id: usize) -> Op {
        self.inc_inst_tracker();
        Op::build_op(
            None,
            None,
            None,
            block_id,
            self.get_inst_num(),
            inst_type,
            &mut self.var_manager,
        )
    }

    pub fn build_op_x(&mut self, x_val: Value, inst_type: InstTy) -> Op {
        self.inc_inst_tracker();
        Op::build_op(
            Some(x_val),
            None,
            None,
            self.get_block_num(),
            self.get_inst_num(),
            inst_type,
            &mut self.var_manager,
        )
    }

    pub fn build_op_x_y(&mut self, x_val: Value, y_val: Value, inst_type: InstTy) -> Op {
        self.inc_inst_tracker();
        Op::build_op(
            Some(x_val),
            Some(y_val),
            None,
            self.get_block_num(),
            self.get_inst_num(),
            inst_type,
            &mut self.var_manager,
        )
    }

    pub fn build_op_x_y_in_block(&mut self, x_val: Value, y_val: Value, inst_type: InstTy, block_id: usize) -> Op {
        self.inc_inst_tracker();
        Op::build_op(
            Some(x_val),
            Some(y_val),
            None,
            block_id,
            self.get_inst_num(),
            inst_type,
            &mut self.var_manager,
        )
    }

    pub fn build_op_y(&mut self, y_val: Value, inst_type: InstTy) -> Op {
        self.inc_inst_tracker();
        Op::build_op(
            None,
            Some(y_val),
            None,
            self.get_block_num(),
            self.get_inst_num(),
            inst_type,
            &mut self.var_manager,
        )
    }

    pub fn build_op_y_in_block(&mut self, y_val: Value, inst_type: InstTy, block_id: usize) -> Op {
        self.inc_inst_tracker();
        Op::build_op(
            None,
            Some(y_val),
            None,
            block_id,
            self.get_inst_num(),
            inst_type,
            &mut self.var_manager,
        )
    }

    pub fn build_spec_op(&mut self, special_val: &String, inst_type: InstTy) -> Op {
        self.inc_inst_tracker();
        Op::build_op(
            None,
            None,
            Some(special_val.clone()),
            self.get_block_num(),
            self.get_inst_num(),
            inst_type,
            &mut self.var_manager,
        )
    }

    pub fn loop_variable_correction(
        &mut self,
        vars: Vec<(Rc<RefCell<UniqueVariable>>, usize)>,
    ) -> Vec<(Rc<RefCell<UniqueVariable>>, usize, usize)> {
        // Grab current_node ID so that we dont alter any uses before the occurrence of this node.
        let current_node = self.graph_manager.clone_node_index();
        let node_starting_point = self.graph_manager.get_node_id(current_node);
        let mut remove_use_vec = Vec::new();
        let mut vars_to_correct = Vec::new();

        let mut local_var_manager = self.var_manager.clone_self();

        // Make map of current graph.
        let mut graph_map = self
            .graph_manager
            .get_mut_ref_graph()
            .node_weights_mut()
            .map(|node| {
                let block_num = node.get_node_id();
                (block_num, node)
            })
            .collect::<HashMap<usize, &mut Node>>();

        // Perform iteration and correction
        vars.iter()
            .filter_map(|(uniq, phi_inst)| match uniq.borrow().get_uses() {
                Some(uses) => Some((uniq, uses, phi_inst)),
                None => None,
            })
            .for_each(|(uniq, uses, phi_inst)| {
                // TODO : Issue being run in to, is that because assignments dont make instructions, they are not updating their phi's correctly.
                // TODO : Need to go through the variable manager and check to see if any matching assignments were made in the block. If so, update them.

                //????
                // TODO : Forgot to add the uses for when build is called....

                //println!("Current Node Id: {}\tPhi Inst: {}", node_starting_point.clone(), phi_inst);
                for (block_num, inst_num) in uses {
                    //println!("Uniq: {}\tBlock: {}\tInst: {}", uniq.borrow().get_ident(), block_num, inst_num);
                    if block_num >= node_starting_point {
                        remove_use_vec.push((Rc::clone(uniq), block_num, inst_num));
                        vars_to_correct.push(Rc::clone(uniq));
                        let node = graph_map
                            .get_mut(&block_num)
                            .expect("Block number should exist");
                        for inst in node.get_mut_data_ref().get_mut_inst_list_ref() {
                            if inst.borrow().get_inst_num() != phi_inst.clone() {
                                let uniq_base = uniq.borrow().get_base_ident();
                                let old_val = Value::new(ValTy::var(Rc::clone(uniq)));
                                let new_val = Value::new(ValTy::var(Rc::clone(
                                    &local_var_manager.get_latest_unique(uniq_base),
                                )));
                                //println!("Inst before: {:?}", inst.borrow());
                                inst.borrow_mut().var_cleanup(old_val, new_val);
                                //println!("Inst after: {:?}", inst.borrow());
                            }
                        }
                    }
                }
            });

        //println!("Uses to Remove: {:?}", remove_use_vec.clone());
        self.var_manager = local_var_manager;

        for uniq in vars_to_correct {
            let uniq_base = uniq.borrow().get_base_ident();
            let old_val = Value::new(ValTy::var(Rc::clone(&uniq)));
            let new_val = Value::new(ValTy::var(Rc::clone(
                &self.var_manager.get_latest_unique(uniq_base),
            )));

            self.var_manager.loop_correction(old_val, new_val);
        }

        remove_use_vec
    }

    /// Graph Specific Functions ///

    pub fn graph_manager(&mut self) -> &mut GraphManager {
        &mut self.graph_manager
    }

    pub fn graph_manager_ref(&self) -> &GraphManager {
        &self.graph_manager
    }

    pub fn new_node(&mut self, node_tag: String, node_type: NodeType) -> &NodeIndex {
        let bt = &mut self.bt;
        self.graph_manager.new_node(node_tag, bt, node_type)
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

    pub fn add_global(&mut self, var: &String) {
        let init_val = Value::new(ValTy::con(0));

        let block_num = self.get_block_num();
        let inst_num = self.get_inst_num();
        self.var_manager
            .add_global(var, init_val, block_num, inst_num);
    }

    pub fn add_variable(&mut self, var: &String) {
        let init_val = Value::new(ValTy::con(0));

        let block_num = self.get_block_num();
        let inst_num = self.get_inst_num();
        self.var_manager
            .add_variable(var, init_val, block_num, inst_num);
    }

    pub fn get_current_unique(&mut self, ident: &String) -> Rc<RefCell<UniqueVariable>> {
        let mut block_num = self.get_block_num();
        let mut inst_num = self.get_inst_num() + 1;
        self.var_manager.get_current_unique(ident.clone())
    }

    pub fn insert_phi_inst(
        &mut self,
        left_set: HashMap<String, Rc<RefCell<UniqueVariable>>>,
        right_set: HashMap<String, Rc<RefCell<UniqueVariable>>>,
    ) -> Vec<(Rc<RefCell<UniqueVariable>>, usize)> {
        let phi_set = VariableManager::build_phi_pairs(left_set, right_set);
        let mut inst_position = 0;
        let mut while_touch_up_vars = Vec::new();

        for (left_var, right_var) in phi_set {
            let block_num = self.get_block_num();
            let inst_num = self.it.get();

            self.var_manager
                .add_var_use(Rc::clone(&left_var), block_num, inst_num + 1);
            self.var_manager
                .add_var_use(Rc::clone(&right_var), block_num, inst_num + 1);

            let left_val = Value::new(ValTy::var(Rc::clone(&left_var)));
            let right_val = Value::new(ValTy::var(Rc::clone(&right_var)));
            let inst = self.build_op_x_y(left_val, right_val, InstTy::phi);
            let inst_val = self.graph_manager.insert_instruction(inst_position, inst);

            // make new unique variable with phi value
            self.var_manager.make_unique_variable(
                left_var.borrow().get_base_ident(),
                inst_val,
                block_num,
                inst_num + 1,
            );

            //while_touch_up_vars.push(left_uniq.clone());
            while_touch_up_vars.push((Rc::clone(&right_var), inst_num + 1));

            inst_position += 1;
        }

        while_touch_up_vars
    }

    /// Array Manager Specific Functions ///

    pub fn array_manager(&mut self) -> &mut ArrayManager {
        &mut self.array_manager
    }

    pub fn build_array_inst(
        &mut self,
        uniq_array: UniqueArray,
        val_vec: Vec<Value>,
        val_to_assign: Option<Value>,
    ) -> Value {
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

    pub fn new_function(&mut self, func_name: String, func_index: &NodeIndex) {
        self.is_func = true;
        let func = self.func_manager.new_function(&func_name, func_index);
        self.array_manager.add_active_function(func.clone());
        self.var_manager.add_active_function(func);
    }

    pub fn end_function(&mut self) -> UniqueFunction {
        self.is_func = false;
        self.var_manager.get_active_function()
    }

    pub fn get_func_call(&mut self, func_name: &String) -> UniqueFunction {
        if self.is_func {
            if func_name.clone() == self.var_manager.active_function().get_name() {
                return self.var_manager.active_function().clone();
            }
        }

        return self.func_manager.get_mut_function(func_name).clone();
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
