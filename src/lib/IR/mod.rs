pub mod ir;
pub mod ir_manager;
pub mod array_manager;
pub mod variable_manager;
pub mod address_manager;
pub mod function_manager;
pub mod ret_register;

use std::rc::Rc;
use std::cell::RefCell;

extern crate petgraph;
use petgraph::Graph;