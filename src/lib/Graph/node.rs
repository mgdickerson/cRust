use lib::Graph::basic_block::BasicBlock;
use lib::IR::ir_manager::{BlockTracker, InstTracker};
use std;

#[derive(Clone)]
pub struct Node {
    node_tag: String,
    node_id: NodeId,
    node_data: NodeData,
    node_type: NodeType,
    node_valid: bool,
}

impl Node {
    pub fn new(node_tag: String, bt: &mut BlockTracker, node_type: NodeType) -> Self {
        let node_data = NodeData::new();
        let node_id = NodeId::new(bt.get());
        bt.increment();
        Node {
            node_tag,
            node_id,
            node_data,
            node_type,
            node_valid: true,
        }
    }

    pub fn get_mut_data_ref(&mut self) -> &mut BasicBlock {
        self.node_data.get_mut_ref()
    }

    pub fn get_data_ref(&self) -> &BasicBlock {
        self.node_data.get_ref()
    }

    pub fn get_data(self) -> BasicBlock {
        self.node_data.get()
    }

    pub fn get_node_id(&self) -> usize {
        self.node_id.get()
    }

    pub fn update_node_id(&mut self, new_node_id: usize) {
        self.node_id = NodeId::new(new_node_id);
    }

    pub fn get_node_type(&self) -> NodeType {
        self.node_type.clone()
    }

    pub fn is_valid(&self) -> bool {
        self.node_valid.clone()
    }

    pub fn mark_node_invalid(&mut self) {
        self.node_valid = false;
    }
}

impl std::fmt::Debug for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "Node: [{}] \\l{} ( \\l {:?}) \\l",
            self.node_id.get(),
            self.node_tag.clone(),
            self.node_data.get_ref()
        )
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum NodeType {
    entrance,
    function_head,
    main_node,
    while_loop_header,
    if_header,
    if_node,
    else_node,
    while_node,
    phi_node,
    bra_node,
    ignored,
    exit,
}

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub struct NodeData {
    data: BasicBlock,
}

impl NodeData {
    pub fn new() -> Self {
        NodeData {
            data: BasicBlock::new(),
        }
    }

    pub fn get(self) -> BasicBlock {
        self.data
    }

    pub fn get_ref(&self) -> &BasicBlock {
        &self.data
    }

    pub fn get_mut_ref(&mut self) -> &mut BasicBlock {
        &mut self.data
    }
}
