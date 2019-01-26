use std;
use lib::IR::basic_block::BasicBlock;
use lib::IR::ir_manager::{InstTracker, BlockTracker};

#[derive(Clone)]
pub struct Node {
    node_id: NodeId,
    node_data: NodeData,
    node_type: NodeType,
}

impl Node {
    pub fn new(it: &mut InstTracker, bt: &mut BlockTracker, node_type: NodeType) -> Self {
        let node_data = NodeData::new(it);
        let node_id = NodeId::new(bt.get());
        Node { node_id, node_data, node_type }
    }

    pub fn get_mut_data_ref(&mut self) -> &mut BasicBlock {
        self.node_data.get_mut_ref()
    }

    pub fn get_data(self) -> BasicBlock {
        self.node_data.get()
    }

    pub fn get_node_id(&self) -> usize {
        self.node_id.get()
    }
}

impl std::fmt::Debug for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Node: [{}] ( \\l {:?}) \\l", self.node_id.get(), self.node_data.get_ref())
    }
}

#[derive(Debug,Clone)]
pub enum NodeType {
    main_node,
    cond_node,
    if_node,
    else_node,
    while_node,
    phi_node,
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
    pub fn new(it: &mut InstTracker) -> Self {
        NodeData { data: BasicBlock::new(it) }
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