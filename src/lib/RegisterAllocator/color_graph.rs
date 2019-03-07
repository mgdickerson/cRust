use petgraph::Graph;
use petgraph::{Directed, Incoming, Outgoing};

use super::{OpNode,Color,RegisterAllocation};
use std::collections::HashMap;
use core::borrow::Borrow;

pub fn color(int_graph: &mut Graph<OpNode, String, Directed, u32>) {
    // Keep track of what has been colored
    let mut walkable_graph = int_graph.clone();
    let mut colored_node_id_map = HashMap::new();

    // Next grab all nodes that have less than 8 edges that can
    // be colored immediately (store the ones that cant in a
    // separate list)
    let mut secondary_color_nodes = Vec::new();

    let mut node_iter = int_graph.node_indices();
    for node_id in node_iter.clone() {
        // TODO : Testing weights
        let weight = int_graph.node_weight(node_id).unwrap().get_weight();
        println!("Inst: {:?} -> Weight: {}", int_graph.node_weight(node_id).unwrap().get_inst_ref(), weight);

        let neighbors = walkable_graph.neighbors_undirected(node_id);
        let num_neighbors = neighbors.clone().count();

        if num_neighbors > 7 {
            secondary_color_nodes.push((node_id,neighbors.clone()));
            continue
        }

        let mut reg_assignment = 1;
        let mut registers_used = Vec::new();

        if colored_node_id_map.is_empty() {
            int_graph.node_weight_mut(node_id)
                .unwrap()
                .assign_register(
                    RegisterAllocation::allocate_register(reg_assignment.clone())
                );
            int_graph.node_weight_mut(node_id)
                .unwrap()
                .get_color();
            colored_node_id_map.insert(node_id, reg_assignment.clone());
        } else {
            let mut neighbor_walker = neighbors.detach();
            while let Some(neighbor_id) = neighbor_walker.next_node(&walkable_graph) {
                let reg_num = int_graph.node_weight(neighbor_id).unwrap().get_register();

                if !registers_used.contains(&reg_num) {
                    registers_used.push(reg_num);
                }
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
        int_graph.node_weight_mut(node_id)
            .unwrap()
            .get_color();
        colored_node_id_map.insert(node_id.clone(), reg_assignment.clone());
    }

    secondary_color_nodes.sort_by_key(|(id, neighbors)| {
        let count = neighbors.clone().count();
        count
    });

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

        int_graph.node_weight_mut(node_id.clone())
            .unwrap()
            .assign_register(
                RegisterAllocation::allocate_register(reg_assignment.clone())
            );
        int_graph.node_weight_mut(node_id.clone())
            .unwrap()
            .get_color();
        colored_node_id_map.insert(node_id.clone(), reg_assignment.clone());
    }
}