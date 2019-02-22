use super::{IRGraphManager, TempValManager, PassId};
use petgraph::prelude::NodeIndex;
use petgraph::algo::has_path_connecting;


pub fn clean_graph(irgm: &mut IRGraphManager, root_node: NodeIndex, temp_manager: &mut TempValManager) {
    // First remove nodes that cannot be reached from the root node.
    let mut walkable_graph = irgm.graph_manager().get_ref_graph().clone();

    // If visit order is sorted by lowest to highest, then reversed, removing nodes should not effect removal of other nodes...
    let mut visit_order = temp_manager.clone_visit_order();
    visit_order.sort_by_key(|index| index.index());
    visit_order.reverse();

    for node_index in visit_order {
        if !has_path_connecting(&walkable_graph, root_node, node_index.clone(), None) {
            println!("Removing node: {}", node_index.index());
            let result = irgm.graph_manager().get_mut_ref_graph().remove_node(node_index);
            match result {
                Some(res) => {},
                None => {
                    println!("Tried to remove, but resulted in None!");

                }
            }
        }
    }
}