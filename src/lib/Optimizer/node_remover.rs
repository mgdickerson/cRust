use super::GraphManager;

pub fn node_removal(graph_manager: &mut GraphManager) {
    for node_index in graph_manager.get_mut_ref_graph().node_indices() {}
}
