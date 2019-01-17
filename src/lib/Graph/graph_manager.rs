use lib::IR::ir::{Value,ValTy,Op,InstTy};
use lib::Graph::node::{Node, NodeId, NodeData};
use lib::IR::ir_manager::IRManager;
extern crate petgraph;
use petgraph::graph::Graph;

pub struct GraphManager {
    graph: Graph<Node, i32>,
    current_node: Node,
}

impl GraphManager {
    pub fn new(graph: Graph<Node,i32>, current_node: Node) -> Self {
        GraphManager { graph, current_node }
    }

    pub fn get_mut_ref_graph(&mut self) -> &mut Graph<Node, i32> {
        &mut self.graph
    }

    pub fn get_graph(self) -> Graph<Node, i32> {
        self.graph
    }

    pub fn add_current_node_to_graph(&mut self) {
        self.graph.add_node(self.current_node.clone());
    }

    pub fn get_mut_ref_current_node(&mut self) -> &mut Node {
        &mut self.current_node
    }

    pub fn get_node(self) -> Node {
        self.current_node
    }

    pub fn update_current_node(&mut self, new_node: Node) {
        self.current_node = new_node;
    }

    pub fn new_node(&mut self, irm: &mut IRManager) -> &mut Node {
        self.current_node = Node::new(irm);
        self.get_mut_ref_current_node()
    }

    pub fn add_instruction(&mut self, inst: Op) {
        self.current_node.get_mut_data_ref().add_instruction(inst);
    }
}