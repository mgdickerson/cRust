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

use super::petgraph::Graph;
use super::{petgraph,graph};



