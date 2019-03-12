use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

pub mod cleaner;
pub mod constant_evaluation;
pub mod cse;
pub mod dce;
pub mod node_remover;
pub mod operator_dominator;
pub mod temp_value_manager;

use lib::IR::address_manager::{AddressManager, UniqueAddress};
use lib::IR::array_manager::{ArrayManager, UniqueArray};
use lib::IR::function_manager::{FunctionManager, UniqueFunction};
use lib::IR::ir::{InstTy, Op, ValTy, Value};
use lib::IR::ir_manager::{BlockTracker, IRGraphManager, InstTracker};
use lib::IR::ret_register::RetRegister;
use lib::IR::variable_manager::{UniqueVariable, VariableManager};

use lib::Graph::basic_block::BasicBlock;
use lib::Graph::graph_manager::GraphManager;
use lib::Graph::node::{Node, NodeData, NodeId, NodeType};
use lib::{clean_base_values, extract_constants};

use self::temp_value_manager::TempValManager;
use super::petgraph::Graph;
use super::{graph, petgraph};
use lib::Optimizer::cleaner::clean_graph;
use petgraph::prelude::NodeIndex;
use petgraph::algo::dominators::simple_fast;
use petgraph::algo::dominators::Dominators;

pub struct Optimizer {
    irgm: IRGraphManager,

    main_temp_val_manager: TempValManager,
    func_temp_val_map: HashMap<String, TempValManager>,
}

impl Optimizer {
    pub fn new(irgm: IRGraphManager) -> Self {
        Optimizer {
            irgm,
            main_temp_val_manager: TempValManager::new(),
            func_temp_val_map: HashMap::new(),
        }
    }

    pub fn get_irgm(self) -> IRGraphManager {
        self.irgm
    }

    pub fn get_main_temp(&self) -> TempValManager {
        self.main_temp_val_manager.clone()
    }

    pub fn get_func_temp(&self) -> HashMap<String, TempValManager> {
        self.func_temp_val_map.clone()
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

        let main_node_index = self.get_irgm_mut_ref().graph_manager().get_main_node();
        extract_constants(&mut self.irgm, main_node_index.clone());

        for (func_name, func_index) in self
            .get_irgm_mut_ref()
            .function_manager()
            .list_functions()
            .iter()
            {
                extract_constants(&mut self.irgm, func_index.clone());
            }

        // Create locals for easier access to them without worrying about borrowing
        let mut local_main_manager = self.get_main_manager();
        let mut local_func_map = self.get_func_manager();

        // Second get temp_val_manager for main
        local_main_manager
            .pull_temp_values(self.get_irgm_mut_ref().graph_manager(), main_node_index);

        let graph_visitor = self
            .irgm
            .graph_manager()
            .graph_visitor(main_node_index.clone());
        for node_id in &graph_visitor {
            if NodeType::exit
                == self
                    .irgm
                    .graph_manager()
                    .get_ref_graph()
                    .node_weight(node_id.clone())
                    .unwrap()
                    .get_node_type()
            {
                constant_evaluation::mark_invalid_nodes(
                    self.get_irgm_mut_ref().graph_manager(),
                    main_node_index,
                    node_id.clone(),
                    &mut local_main_manager,
                );
            }
        }

        // Build managers for all functions in program
        for (func_name, func_index) in self
            .get_irgm_mut_ref()
            .function_manager()
            .list_functions()
            .iter()
        {
            let mut temp_manager = TempValManager::new();
            temp_manager
                .pull_temp_values(self.get_irgm_mut_ref().graph_manager(), func_index.clone());

            let graph_visitor = self.irgm.graph_manager().graph_visitor(func_index.clone());
            for node_id in graph_visitor {
                if NodeType::exit
                    == self
                        .irgm
                        .graph_manager()
                        .get_ref_graph()
                        .node_weight(node_id)
                        .unwrap()
                        .get_node_type()
                {
                    constant_evaluation::mark_invalid_nodes(
                        self.get_irgm_mut_ref().graph_manager(),
                        func_index.clone(),
                        node_id,
                        &mut temp_manager,
                    );
                }
            }

            local_func_map.insert(func_name.clone(), temp_manager);
        }

        // Do some ignored path cleanup
        /*clean_graph(
            &mut self.irgm,
            main_node_index,
            &mut local_main_manager,
            &graph_visitor,
        );*/

        // Return values cloned for locals to update Optimizer
        self.main_temp_val_manager = local_main_manager;
        self.func_temp_val_map = local_func_map;
    }

    pub fn pass_1(&mut self) {
        let mut local_temp_manager = self.main_temp_val_manager.clone();
        let root_node = self.irgm.graph_manager().get_main_node();

        let graph_visitor = self.irgm.graph_manager().graph_visitor(root_node);

        constant_evaluation::eval_program_constants(
            &mut self.irgm,
            &mut local_temp_manager,
            &graph_visitor,
        );

        let new_root = clean_graph(
            &mut self.irgm,
            root_node,
            &mut local_temp_manager,
            &graph_visitor,
        );
        self.irgm.graph_manager().update_main_node(new_root);

        // Return temp manager to itself.
        self.main_temp_val_manager = local_temp_manager;

        for (func, temp_manager) in self.func_temp_val_map.iter_mut() {
            let mut root_node = self
                .irgm
                .function_manager()
                .get_function(func)
                .clone_index();

            // First, update the root node of the function
            for node_id in self.irgm.graph_manager().get_ref_graph().node_indices() {
                let current_node_id = self
                    .irgm
                    .graph_manager()
                    .get_ref_graph()
                    .node_weight(node_id)
                    .unwrap()
                    .get_node_id();

                if root_node.index() == current_node_id {
                    // Found new node_id for function. Update root node and break loop.
                    root_node = node_id;
                    break;
                }
            }

            let function_visitor = self.irgm.graph_manager().graph_visitor(root_node);
            constant_evaluation::eval_program_constants(
                &mut self.irgm,
                temp_manager,
                &function_visitor,
            );

            let new_root = clean_graph(&mut self.irgm, root_node, temp_manager, &function_visitor);
            self.irgm
                .function_manager()
                .get_mut_function(func)
                .update_index(new_root);

            //self.func_temp_val_map.insert(func.clone(), temp_manager.clone());
        }
    }

    pub fn pass_2(&mut self) {
        // Pass 2 consists of CSE
        let root_node = self.irgm.graph_manager().get_main_node();

        cse::trace_common_expression(
            &mut self.irgm,
            &mut self.main_temp_val_manager,
            root_node.clone(),
        );

        let graph = self.irgm.graph_manager().get_mut_ref_graph().clone();
        let dom_space = simple_fast(&graph, root_node.clone());

        let mut load_remover = cse::CLE::new(&mut self.irgm, &root_node, dom_space);
        load_remover.remove_loads(&mut self.irgm, &mut self.main_temp_val_manager);

        let graph_visitor = self.irgm.graph_manager().graph_visitor(root_node.clone());

        let new_root = clean_graph(
            &mut self.irgm,
            root_node,
            &mut self.main_temp_val_manager,
            &graph_visitor,
        );
        self.irgm.graph_manager().update_main_node(new_root);

        for (func_name, temp_manager) in self.func_temp_val_map.iter_mut() {
            let mut root_node = self
                .irgm
                .function_manager()
                .get_function(func_name)
                .clone_index();

            for node_id in self.irgm.graph_manager().get_ref_graph().node_indices() {
                let current_node_id = self
                    .irgm
                    .graph_manager()
                    .get_ref_graph()
                    .node_weight(node_id)
                    .unwrap()
                    .get_node_id();

                if root_node.index() == current_node_id {
                    root_node = node_id;
                    break;
                }
            }

            cse::trace_common_expression(&mut self.irgm, temp_manager, root_node.clone());

            let dom_space = simple_fast(&graph, root_node.clone());

            let mut load_remover = cse::CLE::new(&mut self.irgm, &root_node, dom_space);
            load_remover.remove_loads(&mut self.irgm, temp_manager);

            let function_visitor = self.irgm.graph_manager().graph_visitor(root_node.clone());
            let new_root = clean_graph(&mut self.irgm, root_node, temp_manager, &function_visitor);
            self.irgm
                .function_manager()
                .get_mut_function(func_name)
                .update_index(new_root);
        }
    }

    pub fn pass_3(&mut self) {
        let root_node = self.irgm.graph_manager().get_main_node();

        dce::dead_code_elimination(
            &mut self.irgm,
            &mut self.main_temp_val_manager,
            root_node.clone(),
        );
        let graph_visitor = self.irgm.graph_manager().graph_visitor(root_node.clone());

        let new_root = clean_graph(
            &mut self.irgm,
            root_node,
            &mut self.main_temp_val_manager,
            &graph_visitor,
        );
        self.irgm.graph_manager().update_main_node(new_root);

        for (func_name, temp_manager) in self.func_temp_val_map.iter_mut() {
            let mut func_root_node = self
                .irgm
                .function_manager()
                .get_function(func_name)
                .clone_index();

            for node_id in self.irgm.graph_manager().get_ref_graph().node_indices() {
                let current_node_id = self
                    .irgm
                    .graph_manager()
                    .get_ref_graph()
                    .node_weight(node_id)
                    .unwrap()
                    .get_node_id();

                if func_root_node.index() == current_node_id {
                    func_root_node = node_id;
                    break;
                }
            }

            dce::dead_code_elimination(&mut self.irgm, temp_manager, func_root_node.clone());
            let function_visitor = self
                .irgm
                .graph_manager()
                .graph_visitor(func_root_node.clone());
            let new_root = clean_graph(
                &mut self.irgm,
                func_root_node,
                temp_manager,
                &function_visitor,
            );
            self.irgm
                .function_manager()
                .get_mut_function(func_name)
                .update_index(new_root);
        }
    }
    pub fn pass_4(&mut self) {
        self.irgm.graph_manager().map_blocks_to_node_ids();
    }
}
