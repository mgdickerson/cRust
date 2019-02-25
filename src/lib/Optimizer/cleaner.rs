use super::{IRGraphManager, TempValManager, PassId};
use petgraph::prelude::NodeIndex;
use petgraph::algo::has_path_connecting;
use petgraph::algo::toposort;

pub fn clean_graph(irgm: &mut IRGraphManager, root_node: NodeIndex, temp_manager: &mut TempValManager, graph_visitor: &Vec<NodeIndex>) {
    // First remove nodes that cannot be reached from the root node.
    let mut walkable_graph = irgm.graph_manager().get_ref_graph().clone();

    // If visit order is sorted by lowest to highest, then reversed, removing nodes should not effect removal of other nodes...
    //let mut visit_order = temp_manager.clone_visit_order();
    let mut visit_order = graph_visitor.clone();
    visit_order.sort_by_key(|index| index.index());
    visit_order.reverse();

    // This removes nodes that have been marked (or separated from the main branch)
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

    // Removes inactive instructions from each remaining node
    let visit_order = irgm.graph_manager().graph_visitor(root_node);
    for node_index in visit_order.clone() {
        irgm.graph_manager().remove_inactive_inst(node_index);
    }

    // Removing empty (or useless) nodes.
    // The order of removed nodes is very important as it can shift the numbers below them.
    // Thus reversing the traversal is important.
    let mut reverse_node_visitor = irgm.graph_manager().graph_visitor(root_node);
    reverse_node_visitor.sort_by_key(|node_index| node_index.index());
    reverse_node_visitor.reverse();

    for node_index in reverse_node_visitor {
        println!("Checking Node {:?}", node_index);
        let is_node_valid = irgm.graph_manager().check_node(node_index);

        // Check result of node_check
        if !is_node_valid {
            // Node is not valid, and should be removed. Need to go through
            // the graph and point edges to next node.
            // TODO : Route edges from removed nodes.
            // TODO : this will go through the graph in reverse order of id.
            irgm.graph_manager().get_mut_ref_graph().remove_node(node_index);
        }
        println!("Removed Node {:?}", node_index);
    }

    // Testing purposes, prints out list of currently active instructions
    /*let visit_order = irgm.graph_manager().graph_visitor(root_node);
    for node_index in visit_order {
        for inst in irgm.graph_manager().get_ref_graph().node_weight(node_index).unwrap().get_data_ref().get_inst_list_ref() {
            let inst_id = inst.borrow().get_inst_num();
            let uses = temp_manager.borrow_mut_inst(&inst_id).borrow().active_uses();
            println!("Inst {} has uses: {:?}", inst_id, uses);
        }
    }*/
}