pub mod color_graph;
pub mod spill_handler;


pub mod interference_graph;
use self::interference_graph::{OpNode,RecurseTraverse};

use lib::IR::ir_manager::IRGraphManager;
use std::ffi::OsString;
use std::path::PathBuf;
use std::fs::{self};
use std::fmt::Write;


use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use lib::Graph::node::Node;
use lib::Utility::display;

use super::{petgraph, Graph};
use petgraph::prelude::NodeIndex;
use petgraph::Directed;
use petgraph::algo::dominators::simple_fast;
use lib::RegisterAllocator::color_graph::color;
use lib::Optimizer::temp_value_manager::TempValManager;

pub fn analyze_live_range(
    irgm: &mut IRGraphManager,
    temp_manager: &mut TempValManager,
    root_node: NodeIndex,
    exit_node: NodeIndex,
    path: PathBuf,
    entry: OsString,
) {
    // TODO: Expand to include function calls. (Also still have to fix function returns...)
    // Create a new graph which will contain each instruction as a node,
    // and edges between instructions represent the interference.

    let graph = irgm.graph_manager().get_mut_ref_graph().clone();
    let dom_space = simple_fast(&graph, root_node.clone());

    let mut recurse_graph = RecurseTraverse::new(exit_node, temp_manager,dom_space);

    recurse_graph.recursive_traversal(irgm);
    recurse_graph.coalesce_phis();
    let mut interference_graph = recurse_graph.get_interference_graph();

    color(&mut interference_graph);

    /*for node in interference_graph.node_weights_mut() {
        node.add_color(Color::gray);
    }*/

    let mut dot_graph_path = entry;
    let mut file_name = path.to_str().unwrap().to_owned()
        + "/"
        + dot_graph_path.to_str().unwrap().trim_end_matches(".txt")
        + "_interference.dot";

    let mut output = String::new();
    write!(
        output,
        "{:?}",
        display::Dot::with_config(&interference_graph, &[display::Config::InterferenceGraph])
    );
    fs::write(file_name, output);
}

#[derive(Clone)]
pub enum Color {
    aquamarine,
    peru,
    brown,
    red,
    purple,
    orange,
    green,
    lightblue,

    // spilled register
    gray,
}

impl Color {
    pub fn to_string(&self) -> String {
        match &self {
            // Comes out black
            Color::aquamarine => String::from("aquamarine"),
            // Comes out black
            Color::peru => String::from("peru"),

            // The rest are fine
            Color::brown => String::from("brown"),
            Color::red => String::from("red"),
            Color::purple => String::from("purple"),
            Color::orange => String::from("orange"),
            Color::green => String::from("green"),
            Color::lightblue => String::from("lightblue"),

            // spilled register
            Color::gray => String::from("gray"),
        }
    }

    pub fn get_color(register: & RegisterAllocation) -> Color {
        match register.to_usize() {
            0 => {
                panic!("Registers should not be assigned to 0 in register allocation.");
            },
            1 => {
                Color::aquamarine
            },
            2 => {
                Color::peru
            },
            3 => {
                Color::brown
            },
            4 => {
                Color::red
            },
            5 => {
                Color::purple
            },
            6 => {
                Color::orange
            },
            7 => {
                Color::green
            },
            8 => {
                Color::lightblue
            },
            _ => {
                Color::gray
            },
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct RegisterAllocation {
    reg: usize,
}

impl RegisterAllocation {
    pub fn allocate_register(reg_id: usize) -> Self {
        RegisterAllocation{ reg: reg_id }
    }

    pub fn allocate_R0() -> Self {
        RegisterAllocation { reg: 0 }
    }

    pub fn to_string(&self) -> String {
        String::from("R") + &self.reg.to_string()
    }

    pub fn to_usize(&self) -> usize {
        self.reg.clone()
    }
}
