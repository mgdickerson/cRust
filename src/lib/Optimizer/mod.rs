use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;

pub mod constant_evaluation;
pub mod node_remover;
pub mod temp_value_manager;
pub mod operator_dominator;

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
use lib::clean_graph;

use super::petgraph::Graph;
use super::{petgraph,graph};
use self::temp_value_manager::TempValManager;
use petgraph::prelude::NodeIndex;


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
        clean_graph(self.get_irgm_mut_ref());

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

        // TODO : Maybe add a single pass to remove empty nodes?

        // Return values cloned for locals to update Optimizer
        self.main_temp_val_manager = local_main_manager;
        self.func_temp_val_map = local_func_map;
    }

    pub fn pass_1(&mut self) {
        // For testing I will just use the main branch (later the rest will be added).
        let mut local_temp_manager = self.main_temp_val_manager.clone();
        constant_evaluation::eval_program_constants(&mut self.irgm, &mut local_temp_manager);
    }
}