use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;

pub mod constant_evaluation;
pub mod cse;
pub mod node_remover;
pub mod temp_value_manager;
pub mod operator_dominator;
pub mod cleaner;

use lib::IR::ir_manager::{IRGraphManager, InstTracker, BlockTracker};
use lib::IR::variable_manager::{UniqueVariable,VariableManager};
use lib::IR::function_manager::{UniqueFunction,FunctionManager};
use lib::IR::address_manager::{UniqueAddress,AddressManager};
use lib::IR::array_manager::{UniqueArray,ArrayManager};
use lib::IR::ret_register::RetRegister;
use lib::IR::ir::{Op,Value,ValTy,InstTy};

use lib::Graph::graph_manager::GraphManager;
use lib::Graph::basic_block::BasicBlock;
use lib::Graph::node::{Node,NodeId,NodeData,NodeType};
use lib::clean_base_values;

use super::petgraph::Graph;
use super::{petgraph,graph};
use self::temp_value_manager::TempValManager;
use petgraph::prelude::NodeIndex;
use lib::Optimizer::cleaner::clean_graph;

pub struct Optimizer {
    irgm: IRGraphManager,

    main_temp_val_manager: TempValManager,
    func_temp_val_map: HashMap<String, TempValManager>,
}

impl Optimizer {
    pub fn new(irgm: IRGraphManager) -> Self {
        Optimizer { irgm, main_temp_val_manager: TempValManager::new(), func_temp_val_map: HashMap::new() }
    }

    pub fn get_irgm(self) -> IRGraphManager {
        self.irgm
    }

    pub fn get_irgm_ref(&self) -> &IRGraphManager {
        &self.irgm
    }

    pub fn get_irgm_mut_ref(&mut self) -> &mut IRGraphManager {
        &mut self.irgm
    }

    fn get_main_manager(&self) -> TempValManager {
        self.main_temp_val_manager.clone()
    }

    fn get_func_manager(&self) -> HashMap<String, TempValManager> {
        self.func_temp_val_map.clone()
    }

    pub fn pass_0(&mut self) {
        // First clean graph
        clean_base_values(self.get_irgm_mut_ref());

        // Create locals for easier access to them without worrying about borrowing
        let mut local_main_manager = self.get_main_manager();
        let mut local_func_map = self.get_func_manager();

        // Second get temp_val_manager for main
        let main_node_index = self.get_irgm_mut_ref().graph_manager().get_main_node();
        local_main_manager.pull_temp_values(self.get_irgm_mut_ref().graph_manager(), main_node_index);

        // Build managers for all functions in program
        for (func_name, func_index) in self.get_irgm_mut_ref().function_manager().list_functions().iter() {
            let mut temp_manager = TempValManager::new();
            temp_manager.pull_temp_values(self.get_irgm_mut_ref().graph_manager(), func_index.clone());
            local_func_map.insert(func_name.clone(), temp_manager);
        }

        // Return values cloned for locals to update Optimizer
        self.main_temp_val_manager = local_main_manager;
        self.func_temp_val_map = local_func_map;
    }

    pub fn pass_1(&mut self) {
        let mut local_temp_manager = self.main_temp_val_manager.clone();
        let root_node = self.irgm.graph_manager().get_main_node();

        let graph_visitor = self.irgm.graph_manager().graph_visitor(root_node);

        constant_evaluation::eval_program_constants(&mut self.irgm, &mut local_temp_manager, &graph_visitor);

        let new_root = clean_graph(&mut self.irgm, root_node, &mut local_temp_manager, &graph_visitor);
        self.irgm.graph_manager().update_main_node(new_root);

        // Return temp manager to itself.
        self.main_temp_val_manager = local_temp_manager;

        for (func, temp_manager) in self.func_temp_val_map.iter_mut() {
            let mut root_node = self.irgm.function_manager()
                .get_function(func).clone_index();

            // First, update the root node of the function
            for node_id in self.irgm.graph_manager().get_ref_graph().node_indices() {
                let current_node_id = self.irgm.graph_manager()
                    .get_ref_graph().node_weight(node_id).unwrap()
                    .get_node_id();

                if root_node.index() ==  current_node_id {
                    // Found new node_id for function. Update root node and break loop.
                    root_node = node_id;
                    break;
                }
            }

            let function_visitor = self.irgm.graph_manager().graph_visitor(root_node);
            constant_evaluation::eval_program_constants(&mut self.irgm, temp_manager, &function_visitor);

            let new_root = clean_graph(&mut self.irgm, root_node, temp_manager, &function_visitor);
            self.irgm.function_manager().get_mut_function(func).update_index(new_root);

            //self.func_temp_val_map.insert(func.clone(), temp_manager.clone());
        }
    }

    pub fn pass_2(&mut self) {
        // Pass 2 consists of CSE
        let root_node = self.irgm.graph_manager().get_main_node();
        cse::trace_common_expression(&mut self.irgm, &mut self.main_temp_val_manager, root_node);
    }
}