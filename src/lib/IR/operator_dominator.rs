use lib::IR::ir::{Op,InstTy};
use std::collections::HashMap;
use super::Graph;
use petgraph::graph::NodeIndex;

#[derive(Clone)]
pub struct OpDomHandler {
    op_manager: HashMap<InstTy, OpGraph>,
}

impl OpDomHandler {
    pub fn new() -> Self {
        OpDomHandler { op_manager: HashMap::new() }
    }

    fn get_op_manager(self) -> HashMap<InstTy, OpGraph> {
        self.op_manager
    }

    pub fn get_op_graph(&mut self, op_type: InstTy) -> Option<&mut OpGraph> {
        self.op_manager.get_mut(&op_type)
    }

    pub fn set_recovery_point(&mut self) -> OpDomHandler {
        for (op_type, graph) in self.op_manager.iter_mut() {
            graph.set_recovery_point();
        }

        self.clone()
    }

    pub fn restore(&mut self, op_dom_recovery: OpDomHandler) {
        self.op_manager = op_dom_recovery.get_op_manager();

        for (op_type, graph) in self.op_manager.iter_mut() {
            graph.restore();
        }
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




#[derive(Clone)]
pub struct OpGraph {
    op_graph: Graph<OpNode,i32>,
    head_node: NodeIndex,
    tail_node: NodeIndex,

    recovery_node: Option<NodeIndex>,
    // This is the indicate that when recovery occurred, the first instance
    // of an instruction was created in a non-dominating branch.
    need_new_head: bool,
}

impl OpGraph {
    pub fn new(head_node: OpNode) -> Self {
        let mut op_graph = Graph::new();
        let head_node = op_graph.add_node(head_node);
        let tail_node = head_node.clone();
        OpGraph { op_graph, head_node, tail_node, recovery_node: None, need_new_head: false }
    }

    pub fn clone_tail_index(&self) -> NodeIndex {
        self.tail_node.clone()
    }

    pub fn set_recovery_point(&mut self) {
        let recovery_point = self.tail_node.clone();
        self.recovery_node = Some(recovery_point);
    }

    pub fn restore(&mut self) {
        match self.recovery_node.clone() {
            Some(recovery_point) => {
                self.tail_node = recovery_point;
            },
            None => {
                let recovery_point = self.head_node.clone();
                self.tail_node = recovery_point;
                self.need_new_head = true;
            },
        }
    }

    // True means one was added, should be added to instruction list
    // False means it was found in search, do not add instruction just use value
    pub fn search_or_add(&mut self, new_op: Op) -> (bool, OpNode) {
        let op_tail = self.op_graph.node_weight(self.tail_node).expect("Tail index should have node weight.").clone();

        if self.need_new_head {
            let new_head_op = OpNode::new_head_node(new_op);
            let new_head_node = self.op_graph.add_node(new_head_op.clone());
            self.head_node = new_head_node.clone();
            self.tail_node = new_head_node;
            return (true, new_head_op);
        }

        // check op_tail
        if op_tail.get_op().clone() == new_op {
            return (false, op_tail);
        }

        let mut iter_op = op_tail.clone();

        // Search through Op chain to find matching Op
        while let Some(op_node) = iter_op.next() {
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

    pub fn clone_parent(&self) -> Option<Box<OpNode>> { self.parent_node.clone() }

    pub fn next(&mut self) -> Option<OpNode> {
        match self.parent_node.clone() {
            Some(p_node) => {
                self.op = p_node.clone_op();
                self.parent_node = p_node.clone_parent();
                Some(self.clone())
            },
            None => None,
        }
    }
}