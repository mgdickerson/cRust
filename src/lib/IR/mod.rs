pub mod address_manager;
pub mod array_manager;
pub mod function_manager;
pub mod ir;
pub mod ir_manager;
pub mod ret_register;
pub mod variable_manager;

use std::cell::RefCell;
use std::rc::Rc;

extern crate petgraph;
use petgraph::Graph;
