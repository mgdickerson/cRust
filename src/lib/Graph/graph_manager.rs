use lib::IR::ir::{Value,ValTy,Op,InstTy};
use lib::Graph::node::{Node, NodeId, NodeType, NodeData};
use lib::IR::ir_manager::{InstTracker, BlockTracker};
extern crate petgraph;
use petgraph::graph::Graph;
use petgraph::prelude::NodeIndex;

pub struct GraphManager {
    graph: Graph<Node, i32>,
    current_node_index: NodeIndex,
}

impl GraphManager {
    pub fn new(mut graph: Graph<Node,i32>, it: &mut InstTracker, bt: &mut BlockTracker) -> Self {
        let current_node = Node::new(it,bt, NodeType::main_node);
        let current_node_index = graph.add_node(current_node);
        GraphManager { graph, current_node_index }
    }

    // -- Node Related Functions -- //

    pub fn new_node(&mut self, it: &mut InstTracker, bt: &mut BlockTracker, node_type: NodeType) -> &mut NodeIndex {
        let current_node = Node::new(it, bt, node_type);
        self.current_node_index = self.graph.add_node(current_node);
        self.get_mut_ref_current_node_index()
    }

    pub fn get_node_index(self) -> NodeIndex {
        self.current_node_index
    }

    pub fn clone_node_index(&self) -> NodeIndex { self.current_node_index.clone() }

    pub fn get_mut_ref_current_node_index(&mut self) -> &mut NodeIndex {
        &mut self.current_node_index
    }

    pub fn switch_current_node_index(&mut self, new_node: NodeIndex) {
        self.current_node_index = new_node;
    }

    pub fn get_node_id(&self, node_index: NodeIndex) -> usize {
        self.graph.node_weight(node_index)
            .expect("Expected node weight for node_id")
            .get_node_id()
    }

    /* Dont think i need this one.
    pub fn add_node_to_graph(&mut self, node: Node) -> NodeIndex {
        self.graph.add_node(node)
    }
    */

    pub fn add_edge(&mut self, parent: NodeIndex, child: NodeIndex) {
        self.graph.add_edge(parent, child, 1);
    }

    // -- Graph Related Functions -- //

    pub fn get_mut_ref_graph(&mut self) -> &mut Graph<Node, i32> {
        &mut self.graph
    }

    pub fn get_graph(self) -> Graph<Node, i32> {
        self.graph
    }

    // -- Convenience Feature for adding inst -- //

    pub fn add_instruction(&mut self, inst: Op) {
        self.graph.node_weight_mut(self.current_node_index)
            .expect("Expected Node to have weight, none was found while adding instruction.")
            .get_mut_data_ref()
            .add_instruction(inst);
    }
}