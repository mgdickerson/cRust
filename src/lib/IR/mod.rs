pub mod ir;
pub mod basic_block;
pub mod ir_manager;
pub mod array_manager;
pub mod variable_manager;
pub mod address_manager;
pub mod operator_dominator;
pub mod function_manager;

extern crate petgraph;
use petgraph::Graph;