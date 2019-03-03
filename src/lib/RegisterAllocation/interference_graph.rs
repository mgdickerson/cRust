use std::rc::Rc;
use std::cell::RefCell;

use super::{Node,NodeIndex};
use lib::IR::ir::Op;
use lib::IR::ir_manager::IRGraphManager;

use petgraph::Graph;
use petgraph::{Outgoing,Incoming, Directed};
use petgraph::algo::dominators::Dominators;
use petgraph::algo::dominators::simple_fast;

pub struct InterferenceGraph {
    inter_graph: Graph<Node,String,Directed,u32>,
}

impl InterferenceGraph {

}

pub fn analyze_live_range(irgm: &mut IRGraphManager,
                      //inter_graph: &mut Graph<Node,String,Directed,u32>,
                      root_node: NodeIndex) {
    // Make vector of live instructions.
    // When a new instruction is found that is not
    // part of the "live" instructions, add it to
    // the list and add an edge to it to all other
    // live instructions.
    //let mut live_values = Vec::new();

    // Create a new graph which will contain each instruction as a node,
    // and edges between instructions represent the interference.
    let interference_graph : Graph<Rc<RefCell<Op>>, String, Directed, u32> = Graph::new();

    let graph = irgm.graph_manager().get_mut_ref_graph().clone();
    let dom_space = simple_fast(&graph,root_node.clone());

    let mut visited = Vec::new();
    let final_node = recurse_base_node(irgm, root_node, &mut visited);

    // TODO : No longer need recurse base, just use the exit points
    // on the graphs provided. Those should be the correct exit points
    // for the graph and thus should work for bottom up traversal.

}

/// Recursing options:
/// 0-Parents                    -> End of recursion, return completed live range
/// 1-Parent                     -> Continue directly up graph
/// 2-Parents (neither dominate) -> An if branch. go up left (until loop header is found), then traverse up right.
///                                 Continue with the loop header after joining live range from left and right.
/// 2-Parents (one dominates)    -> An else branch, loop through the branch dominated by loop twice, then traverse
///                                 up the dominating path.
fn recurse_graph(irgm: &mut IRGraphManager,
                 current_node: NodeIndex,
                 live_range: &mut Vec<NodeIndex>,
                 dominators: & Dominators<NodeIndex>,
                 loop_break_point: Option<NodeIndex>) {
    // Ensure that the while loops dont recurse through the graph infinitely
    if let Some(node_id) = loop_break_point.clone() {
        if current_node == current_node {
            return
        }
    }

    // Get parents from current node.
    let mut parents = Vec::new();
    println!("Children of node: {:?}", current_node.clone());
    for parent_id in irgm.graph_manager().get_ref_graph().neighbors_directed(current_node.clone(), Incoming) {
        parents.push(parent_id);
        println!("Child node: {:?}", parent_id);
    }

    match parents.len() {
        0 => {
            // Final node, perform any required actions then simply return.
            return
        },
        1 => {
            let current_node = parents.pop().unwrap();
            recurse_graph(irgm, current_node, live_range, dominators, loop_break_point);
            return
        },
        2 => {
            // two cases here, if or while
            let mut ordered_parents = Vec::new();
            let mut is_while = false;

            // This gives both information as to which control flow type it
            // is, as well as sorting for the while case.
            for node_id in parents.iter() {
                if dominators.immediate_dominator(current_node) == Some(node_id.clone()) {
                    ordered_parents.insert(0, node_id.clone());
                    is_while = true;
                } else {
                    ordered_parents.push(node_id.clone());
                }
            }

            if is_while {
                // This path will require two runs through the non-dominating path.
                // 0 is the dominating path, thus goes second.
                // 1 is the looping path, thus must go through it twice.

                recurse_graph(irgm,
                              ordered_parents[1].clone(),
                              live_range,
                              dominators,
                              Some(current_node.clone()));
                recurse_graph(irgm,
                              ordered_parents[1].clone(),
                              live_range,
                              dominators,
                              Some(current_node.clone()));
                recurse_graph(irgm, ordered_parents[0].clone(), live_range, dominators, None);
                return
            } else {
                // This is the if case. Traverse up both paths until the dominator is hit, then return
                // and merge the two live ranges and go through the dominating path.
                let immediate_dominator = dominators.immediate_dominator(current_node).unwrap().clone();
                recurse_graph(irgm, ordered_parents[0].clone(), live_range, dominators, Some(immediate_dominator.clone()));
                recurse_graph(irgm, ordered_parents[1].clone(), live_range, dominators, Some(immediate_dominator.clone()));

                // Combine live ranges here.
                // TODO : ^

                recurse_graph(irgm, immediate_dominator, live_range, dominators, None);
                return
            }
        },
        _ => {
            panic!("Should be no more than 2 parents for any given node of the graph.");
        }
    }
}

fn recurse_base_node(irgm: & IRGraphManager, test_node: NodeIndex, visited: &mut Vec<NodeIndex>) -> Option<NodeIndex> {
    visited.push(test_node);
    let mut neighbors = irgm.graph_manager_ref().get_ref_graph().neighbors_directed(test_node, Outgoing);

    let mut children = Vec::new();
    while let Some(child) = neighbors.next() {
        children.push(child);
    }
    children.reverse();

    if children.len() == 0 {
        return Some(test_node)
    } else {
        for child in children.iter() {
            if visited.contains(&child) {
                continue
            }
            let ret_node = recurse_base_node(irgm, child.clone(), visited);
            match ret_node {
                Some(node_id) => return Some(node_id),
                None => {
                    // loop again
                },
            }
        }
    }

    // If path reaches this point, then this was not the path to the final node.
    // If no path reaches a node with no children, it is an infinite loop.
    None
}