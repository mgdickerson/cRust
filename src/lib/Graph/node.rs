use std;
use lib::IR::ir::Inst;
use lib::Graph::arena::Arena;

pub struct Node {
    parents: Option<Vec<NodeId>>,
    children: Option<Vec<NodeId>>,
    node_id: NodeId,

    instructions: Vec<Box<dyn Inst>>
}

impl Node {
    pub fn new(node_id: NodeId) -> Self {
        Node { parents: None, children: None, node_id, instructions: vec!() }
    }

    pub fn parents(&self) -> &Option<Vec<NodeId>> {
        &self.parents
    }

    pub fn add_parent(&mut self, new_parent: NodeId) {
        match self.parents.as_mut() {
            Some(nodes) => {
                nodes.push(new_parent);
                return
            },
            None => {
                // fall through
            },
        }

        // There is no vector of Parents, create new one.
        let mut parents = vec!();
        parents.push(new_parent);
        self.parents = Some(parents);
    }

    pub fn children(&self) -> &Option<Vec<NodeId>> {
        &self.children
    }

    pub fn add_child(&mut self, new_child: NodeId) {
        match self.children.as_mut() {
            Some(nodes) => {
                nodes.push(new_child);
                return
            },
            None => {
                // fall through
            },
        }

        let mut children = vec!();
        children.push(new_child);
        self.children = Some(children);
    }

    pub fn instructions(&self) -> &Vec<Box<Inst>> {
        &self.instructions
    }

    pub fn add_instr(&mut self, new_inst: Box<Inst>) {
        self.instructions.push(new_inst)
    }

    pub fn node_id(&self) -> NodeId {
        self.node_id.clone()
    }
}

#[derive(Debug,Clone)]
pub struct NodeId {
    index: usize,
}

impl NodeId {
    pub fn new(index: usize) -> Self {
        Self { index }
    }

    pub fn get(&self) -> usize {
        self.index.clone()
    }
}