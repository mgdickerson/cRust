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

    let visit_order = irgm.graph_manager().graph_visitor(root_node);
    for node_index in visit_order.clone() {
        irgm.graph_manager().remove_inactive_inst(node_index);
    }

    for node_index in visit_order {
        for inst in irgm.graph_manager().get_ref_graph().node_weight(node_index).unwrap().get_data_ref().get_inst_list_ref() {
            let inst_id = inst.borrow().get_inst_num();
            let uses = temp_manager.borrow_mut_inst(&inst_id).borrow().active_uses();
            println!("Inst {} has uses: {:?}", inst_id, uses);
        }
    }
}