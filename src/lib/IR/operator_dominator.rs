use lib::IR::ir::{Op,InstTy};
use std::collections::HashMap;
use super::Graph;
use petgraph::graph::NodeIndex;

pub struct OpDomHandler {
    op_manager: HashMap<InstTy, OpGraph>,
}

impl OpDomHandler {
    pub fn new() -> Self {
        OpDomHandler { op_manager: HashMap::new() }
    }

    pub fn get_op_graph(&mut self, op_type: InstTy) -> Option<&mut OpGraph> {
        self.op_manager.get_mut(&op_type)
    }

    // True means new one was added, should be added to instruction list
    // False means it was found in search, do not add instruction just use value
    pub fn search_or_add_inst(&mut self, new_op: Op) -> (bool, Op) {
        let contains_key = self.op_manager.contains_key(new_op.inst_type());

        if !contains_key {
            let key = new_op.inst_type().clone();
            let op_head = OpNode::new_head_node(new_op.clone());

            self.op_manager.insert(key, OpGraph::new(op_head));

            return (true, new_op);
        }

        let (is_new, op_node) = self.op_manager.get_mut(new_op.inst_type())
            .expect("Key is present, should have graph.")
            .search_or_add(new_op);

        (is_new, op_node.get_op().clone())
    }
}




pub struct OpGraph {
    op_graph: Graph<OpNode,i32>,
    head_node: NodeIndex<u32>,
    tail_node: NodeIndex,
}

impl OpGraph {
    pub fn new(head_node: OpNode) -> Self {
        let mut op_graph = Graph::new();
        let head_node = op_graph.add_node(head_node);
        let tail_node = head_node.clone();
        OpGraph { op_graph, head_node, tail_node }
    }

    pub fn clone_tail_index(&self) -> NodeIndex {
        self.tail_node.clone()
    }

    pub fn revert_tail_index(&mut self, restore_index: NodeIndex) {
        self.tail_node = restore_index;
    }

    // True means one was added, should be added to instruction list
    // False means it was found in search, do not add instruction just use value
    pub fn search_or_add(&mut self, new_op: Op) -> (bool, OpNode) {
        let op_tail = self.op_graph.node_weight(self.tail_node).expect("Tail index should have node weight.").clone();

        // check op_tail
        if op_tail.get_op().clone() == new_op {
            return (false, op_tail);
        }

        // Search through Op chain to find matching Op
        while let Some(op_node) = op_tail.get_parent() {
            if op_node.get_op().clone() == new_op.clone() {
                return (false, op_node.clone());
            }
        }

        // No Op found, add this Op to Op-Chain and return
        let new_tail = OpNode::add_op_node(new_op, op_tail);
        self.tail_node = self.add_child_op(new_tail.clone());

        (true, new_tail)
    }

    pub fn add_child_op(&mut self, child_op: OpNode) -> NodeIndex<u32> {
        let child_node = self.op_graph.add_node(child_op);
        self.op_graph.add_edge(self.tail_node,child_node.clone(), 1);

        return child_node;
    }

    pub fn get_graph(&self) -> &Graph<OpNode, i32> {
        &self.op_graph
    }
}

#[derive(Clone)]
pub struct OpNode {
    op: Op,
    parent_node: Option<Box<OpNode>>,
}

impl OpNode {
    pub fn new_head_node(op_head: Op) -> Self {
        OpNode { op: op_head, parent_node: None }
    }

    pub fn add_op_node(op: Op, parent_op: OpNode) -> Self {
        OpNode { op, parent_node: Some(Box::new(parent_op)) }
    }

    pub fn get_op(&self) -> &Op {
        &self.op
    }

    pub fn clone_op(&self) -> Op {
        self.op.clone()
    }

    pub fn get_parent(&self) -> Option<OpNode> {
        match self.parent_node.clone() {
            Some(p_node) => {
                Some(*p_node)
            },
            None => None,
        }
    }
}