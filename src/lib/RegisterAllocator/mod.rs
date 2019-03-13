pub mod color_graph;
pub mod spill_handler;

pub mod interference_graph;
use self::interference_graph::{OpNode, RecurseTraverse};

use lib::IR::ir_manager::IRGraphManager;
use std::ffi::OsString;
use std::fmt::Write;
use std::fs;
use std::path::PathBuf;

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use lib::Graph::node::Node;
use lib::Utility::display;

use super::{petgraph, Graph};
use lib::Optimizer::temp_value_manager::TempValManager;
use lib::RegisterAllocator::color_graph::color;
use lib::RegisterAllocator::spill_handler::SpillHandler;
use petgraph::algo::dominators::simple_fast;
use petgraph::prelude::NodeIndex;
use petgraph::Directed;

pub fn analyze_live_range(
    irgm: &mut IRGraphManager,
    temp_manager: &mut TempValManager,
    root_node: NodeIndex,
    exit_nodes: Vec<NodeIndex>,
    func_name: Option<String>,
    path: PathBuf,
    entry: OsString,
) -> HashMap<usize, usize> {
    // Create a new graph which will contain each instruction as a node,
    // and edges between instructions represent the interference.

    let graph = irgm.graph_manager().get_mut_ref_graph().clone();
    let dom_space = simple_fast(&graph, root_node.clone());
    let mut spill_handler = SpillHandler::new();

    let mut needs_coloring = true;
    let mut spilled_instructions = HashMap::new();

    let mut round_count = 0;
    let mut inst_register_map = HashMap::new();

    temp_manager.pull_temp_values(irgm.graph_manager(), root_node);

    while needs_coloring {
        //println!("Round: {}", round_count);
        round_count += 1;
        let mut recurse_graph = RecurseTraverse::new(root_node, temp_manager, dom_space.clone());

        for exit_node_id in exit_nodes.clone().iter() {
            recurse_graph.set_starting_exit(exit_node_id.clone());
            recurse_graph.recursive_traversal(irgm, &spilled_instructions);
        }

        recurse_graph.coalesce_phis();

        let mut interference_graph = recurse_graph.get_interference_graph();

        // It appears to have stabilized, though it very consistently
        // drops about 3 instructions on 24, which seems like a lot
        let color_result = color(&mut interference_graph);

        match color_result {
            Ok(_) => {
                needs_coloring = false;

                let mut dot_graph_path = entry.clone();
                let mut file_name = path.to_str().unwrap().to_owned()
                    + "/"
                    + dot_graph_path.to_str().unwrap().trim_end_matches(".txt");

                if let Some(func_name) = &func_name {
                    file_name += func_name;
                }

                file_name += "_interference.dot";

                let mut output = String::new();
                write!(
                    output,
                    "{:?}",
                    display::Dot::with_config(
                        &interference_graph,
                        &[display::Config::InterferenceGraph]
                    )
                );
                fs::write(file_name, output);

                // TODO : return mapping of instructions to registers
                let node_indicies = interference_graph.node_indices().clone();
                for node_id in node_indicies {
                    let register = interference_graph
                        .node_weight(node_id.clone())
                        .unwrap()
                        .get_register();
                    for op_inst in interference_graph
                        .node_weight(node_id)
                        .unwrap()
                        .get_inst_ref()
                        .clone()
                    {
                        inst_register_map.insert(op_inst.borrow().get_inst_num(), register.clone());
                    }
                }
            }
            Err(spill_node) => {
                //println!("Splitting instruction: {:?}", interference_graph.node_weight(spill_node)
                //    .unwrap().get_inst_ref()[0]);
                let inst_id = interference_graph
                    .node_weight(spill_node)
                    .unwrap()
                    .get_inst_ref()[0]
                    .borrow()
                    .get_inst_num();
                spill_handler.spill_value(irgm, temp_manager, inst_id.clone());
                if !spilled_instructions.contains_key(&inst_id) {
                    spilled_instructions.insert(inst_id, 1);
                } else {
                    let spill_num = spilled_instructions.get_mut(&inst_id).unwrap().clone();
                    spilled_instructions.insert(inst_id, spill_num + 1);
                }
                temp_manager.pull_temp_values(irgm.graph_manager(), root_node);
            }
        }
    }

    inst_register_map
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

    pub fn get_color(register: &RegisterAllocation) -> Color {
        match register.to_usize() {
            0 => {
                panic!("Registers should not be assigned to 0 in register allocation.");
            }
            1 => Color::aquamarine,
            2 => Color::peru,
            3 => Color::brown,
            4 => Color::red,
            5 => Color::purple,
            6 => Color::orange,
            7 => Color::green,
            8 => Color::lightblue,
            _ => Color::gray,
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct RegisterAllocation {
    reg: usize,
}

impl RegisterAllocation {
    pub fn allocate_register(reg_id: usize) -> Self {
        RegisterAllocation { reg: reg_id }
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
