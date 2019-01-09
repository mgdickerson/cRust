use lib::IR::ir::Inst;

pub struct Node {
    parents: Option<Vec<NodeId>>,
    children: Option<Vec<NodeId>>,

    instructions: Vec<Box<dyn Inst>>
}

impl Node {
    pub fn parents(&self) -> &Option<Vec<NodeId>> {
        &self.parents
    }

    pub fn children(&self) -> &Option<Vec<NodeId>> {
        &self.children
    }

    pub fn instructions(&self) -> &Vec<Box<Inst>> {
        &self.instructions
    }
}

pub struct NodeId {
    index: usize,
}

impl NodeId {
    pub fn new(index: usize) -> Self {
        Self { index }
    }


}