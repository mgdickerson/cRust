use super::{Node,Directed,NodeIndex};

use petgraph::Graph;
use lib::IR::ir_manager::IRGraphManager;

pub struct InterferenceGraph {
    inter_graph: Graph<Node,String,Directed,u32>,
}

impl InterferenceGraph {

}

fn analyze_live_range(irgm: &mut IRGraphManager, inter_graph: &mut Graph<Node,String,Directed,u32>, root_node: NodeIndex) {
    // Make vector of live instructions.
    // When a new instruction is found that is not
    // part of the "live" instructions, add it to
    // the list and add an edge to it to all other
    // live instructions.
    let mut live_values = Vec::new();

    let mut visit_order = irgm.graph_manager().graph_visitor(root_node.clone());
    let final_node = visit_order.pop().expect("Returned a visit order with no nodes in it.");


}