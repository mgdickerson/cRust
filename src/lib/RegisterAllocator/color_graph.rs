use petgraph::Graph;
use petgraph::{Directed, Incoming, Outgoing};

use super::{OpNode,Color,RegisterAllocation};
use std::collections::HashMap;
use petgraph::prelude::NodeIndex;

pub fn color(int_graph: &mut Graph<OpNode, String, Directed, u32>) -> Result<(), NodeIndex> {
    // Keep track of what has been colored
    let mut walkable_graph = int_graph.clone();

    // Next grab all nodes that have less than 8 edges that can
    // be colored immediately (store the ones that cant in a
    // separate list)

    // Grab the weights of all nodes and their index_id,
    // then sort by weight in decending order
    let mut initial_sort = walkable_graph
        .node_indices()
        .map(|node_id| {
            let weight = int_graph.node_weight(node_id).unwrap().get_weight();
            (node_id.clone(), weight)
        }).collect::<Vec<(NodeIndex,usize)>>();
    initial_sort.sort_by_key(|(node_id,weight)| {
        weight.clone()
    });
    initial_sort.reverse();

    // Anything that cant be sorted by the initial greedy sort will
    // be sorted in a second pass in order of weight.
    let mut secondary_color_nodes = Vec::new();

    for (node_id, _) in initial_sort {
        // Grab number of neighbors
        let neighbors = walkable_graph.neighbors_undirected(node_id);
        let num_neighbors = neighbors.clone().count();

        // If it has more than 7 neighbors it cant be immediately colored
        if num_neighbors > 7 {
            secondary_color_nodes.push((node_id,neighbors.clone()));
            continue
        }

        let mut reg_assignment = 1;
        let mut registers_used = Vec::new();


        let mut neighbor_walker = neighbors.detach();
        while let Some(neighbor_id) = neighbor_walker.next_node(&walkable_graph) {
            let reg_num = int_graph.node_weight(neighbor_id).unwrap().get_register();

            if !registers_used.contains(&reg_num) {
                registers_used.push(reg_num);
            }
        }

        while registers_used.contains(&reg_assignment) {
            reg_assignment += 1;
        }

        int_graph.node_weight_mut(node_id)
            .unwrap()
            .assign_register(
                RegisterAllocation::allocate_register(reg_assignment.clone())
            );
    }

    for (node_id, neighbors) in secondary_color_nodes.iter() {
        let mut reg_assignment = 1;
        let mut registers_used = Vec::new();

        let mut neighbor_walker = neighbors.detach();
        while let Some(neighbor_id) = neighbor_walker.next_node(&walkable_graph) {
            let reg_num = int_graph.node_weight(neighbor_id).unwrap().get_register();

            if !registers_used.contains(&reg_num) {
                registers_used.push(reg_num);
            }
        }

        while registers_used.contains(&reg_assignment) {
            reg_assignment += 1;
        }

        if reg_assignment > 8 {
            // This value is above register limit, needs to be spilled.
            // Return spilled value.
            return Err(node_id.clone())
        }

        int_graph.node_weight_mut(node_id.clone())
            .unwrap()
            .assign_register(
                RegisterAllocation::allocate_register(reg_assignment.clone())
            );
    }

    for node in int_graph.node_weights_mut() {
        node.get_color();
    }

    Ok(())
}