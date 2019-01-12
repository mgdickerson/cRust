use std;
use lib::IR::basic_block::BasicBlock;

#[derive(Clone)]
pub struct Node {
    node_id: NodeId,
    node_data: NodeData,
}

impl Node {
    pub fn new(node_id: NodeId) -> Self {
        Node { node_id, node_data: NodeData::new() }
    }

    pub fn get_mut_data_ref(&mut self) -> &mut BasicBlock {
        self.node_data.get_mut_ref()
    }

    pub fn get_data(self) -> BasicBlock {
        self.node_data.get()
    }
}

impl std::fmt::Debug for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Node: [{}] ( \\l {:?}) \\l", self.node_id.get(), self.node_data.get_ref())
    }
}

#[derive(Debug,Clone)]
pub struct NodeId {
    unique_id: usize,
}

impl NodeId {
    pub fn new(unique_id: usize) -> Self {
        NodeId { unique_id }
    }

    pub fn get(&self) -> usize {
        self.unique_id.clone()
    }
}

#[derive(Debug,Clone)]
pub struct NodeData {
    data: BasicBlock,
}

impl NodeData {
    pub fn new() -> Self {
        NodeData { data: BasicBlock::new() }
    }

    pub fn get(self) -> BasicBlock {
        self.data
    }

    pub fn get_ref(& self) -> &BasicBlock {
        & self.data
    }

    pub fn get_mut_ref(&mut self) -> &mut BasicBlock {
        &mut self.data
    }
}