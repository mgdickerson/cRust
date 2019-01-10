use lib::Graph::node::{Node, NodeId};

pub struct Arena {
    nodes: Vec<Node>,
    node_id_count: usize,
}

impl Arena {
    pub fn new() -> Self {
        Arena { nodes: Vec::new(), node_id_count: 0 }
    }

    pub fn new_node(&mut self) -> NodeId {
        // make a new NodeId, then increment node_id_count
        let new_node_id = NodeId::new(self.node_id_count);
        self.node_id_count += 1;

        // make new Node with new_node_id, push to nodes
        self.nodes.push(Node::new(new_node_id.clone()));

        // return NodeId
        new_node_id
    }

    pub fn count(&self) -> usize {
        self.nodes.len()
    }

    pub fn get_mut_ref(&mut self, id: NodeId) -> Option<&mut Node> {
        self.nodes.get_mut(id.get())
    }

    pub fn iter(&self) -> std::slice::Iter<Node> {
        self.nodes.iter()
    }
}