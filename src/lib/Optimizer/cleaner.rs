use super::{IRGraphManager, TempValManager};
use petgraph::algo::has_path_connecting;
use petgraph::algo::toposort;
use petgraph::prelude::NodeIndex;
use petgraph::{Directed, Incoming, Outgoing};

pub fn clean_graph(
    irgm: &mut IRGraphManager,
    root_node: NodeIndex,
    temp_manager: &mut TempValManager,
    graph_visitor: &Vec<NodeIndex>,
) -> NodeIndex {
    let mut return_node_id = root_node.clone();
    let mut new_root_id = root_node.index();

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
            //println!("Removing node: {}", node_index.index());
            let result = irgm
                .graph_manager()
                .get_mut_ref_graph()
                .remove_node(node_index);
            match result {
                Some(res) => {}
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
        //println!("Checking Node {:?}", node_index);
        let is_node_valid = irgm.graph_manager().check_node(node_index);

        // Check result of node_check
        if !is_node_valid {
            // Node is not valid, and should be removed. Need to go through
            // the graph and point edges to next node.

            // First check all edges incoming
            let mut parents = Vec::new();
            //println!("Children of node: {:?}", node.clone());
            for parent_id in irgm
                .graph_manager()
                .get_ref_graph()
                .neighbors_directed(node_index.clone(), Incoming)
            {
                parents.push(parent_id);
            }

            // Check all outgoing edges
            let mut children = Vec::new();

            for child_id in irgm
                .graph_manager()
                .get_ref_graph()
                .neighbors_directed(node_index.clone(), Outgoing)
            {
                children.push(child_id);
            }

            // Bridge the parent nodes to the child nodes.
            for parent in &parents {
                for child in &children {
                    if !irgm
                        .graph_manager()
                        .get_ref_graph()
                        .contains_edge(parent.clone(), child.clone())
                    {
                        irgm.graph_manager().add_edge(parent.clone(), child.clone());
                    }
                }
            }

            // If the main root_node is being removed, a new return value will be needed
            if node_index == root_node.clone() {
                if !children.is_empty() {
                    // Ensure that children is not empty and use first node for return_id
                    //println!("Children[0]: {:?}", children[0]);
                    new_root_id = irgm
                        .graph_manager()
                        .get_ref_graph()
                        .node_weight(children[0].clone())
                        .unwrap()
                        .get_node_id();
                }
            }

            // Remove no longer used node.
            irgm.graph_manager()
                .get_mut_ref_graph()
                .remove_node(node_index);
        }
        //println!("Removed Node {:?}", node_index);
    }

    // Update branch commands
    // TODO : Update branch commands.

    // Remove inactive nodes in reverse order
    let mut node_vec = irgm
        .graph_manager()
        .get_ref_graph()
        .node_indices()
        .collect::<Vec<NodeIndex>>();
    node_vec.sort_by_key(|id| id.clone());
    node_vec.reverse();

    // This should remove all the invalid nodes.
    for node_id in node_vec {
        if !irgm
            .graph_manager()
            .get_ref_graph()
            .node_weight(node_id)
            .unwrap()
            .is_valid()
        {
            irgm.graph_manager()
                .get_mut_ref_graph()
                .remove_node(node_id);
        }

        /*// renumber all node_index
        irgm.graph_manager()
            .get_mut_ref_graph()
            .node_weight_mut(node_id.clone())
            .unwrap()
            .update_node_id(node_id.index());*/
    }

    // Using new_root_id to look up actual location (NodeIndex)
    for node_id in irgm.graph_manager().get_ref_graph().node_indices() {
        let current_node_id = irgm
            .graph_manager()
            .get_ref_graph()
            .node_weight(node_id)
            .unwrap()
            .get_node_id();

        if new_root_id == current_node_id {
            // Found new node_id for function. Update root node and break loop.
            return_node_id = node_id;
            break;
        }
    }

    //println!("Sending back new main node: {:?}", return_node_id);
    return_node_id
}
