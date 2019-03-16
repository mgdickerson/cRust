// Traverse through the graph in correct traversal order
// and put all instructions in order into a single vector.
use lib::IR::ir_manager::IRGraphManager;

use petgraph::prelude::{NodeIndex};
use petgraph::{Directed, Incoming, Outgoing};

pub struct CodeGen {

}

impl CodeGen {
    pub fn new() -> Self {
        CodeGen {}
    }
}

fn traversal_path(irgm: &mut IRGraphManager, current_node: NodeIndex, visited: &mut Vec<NodeIndex>) {
    if visited.contains(&current_node) {
        return
    }

    let mut children = irgm.graph_manager()
        .get_ref_graph()
        .neighbors_directed(current_node.clone(), Outgoing)
        .detach();

    let node_type = irgm.graph_manager()
        .get_ref_graph()
        .node_weight(current_node)
        .unwrap()
        .get_node_type();

    
}