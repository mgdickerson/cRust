use lib::IR::ir::{InstTy, Op};

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use super::Graph;
use petgraph::algo::dominators::Dominators;
use petgraph::graph::NodeIndex;
use std::any::Any;

#[derive(Clone)]
pub struct OpDomHandler {
    op_manager: HashMap<InstTy, HashMap<Op, Vec<OpValue>>>,
}

impl OpDomHandler {
    pub fn new() -> Self {
        OpDomHandler {
            op_manager: HashMap::new(),
        }
    }

    fn get_op_manager(self) -> HashMap<InstTy, HashMap<Op, Vec<OpValue>>> {
        self.op_manager
    }

    /*pub fn get_op_graph(&mut self, op_type: InstTy) -> Option<&mut OpNode> {
        self.op_manager.get_mut(&op_type)
    }*/

    // Dominance path

    // True means new one was added, should be added to instruction list
    // False means it was found in search, do not add instruction just use value
    pub fn search_or_add_inst(
        &mut self,
        next_op: Rc<RefCell<Op>>,
        node_id: NodeIndex,
        dom_space: & Dominators<NodeIndex>,
    ) -> (bool, Rc<RefCell<Op>>) {
        let key = next_op.borrow().inst_type().clone();
        let contains_key = self.op_manager.contains_key(&key);

        if !contains_key {
            let mut new_inst_hash = HashMap::new();
            let mut new_inst_vec = Vec::new();

            let new_op_value = OpValue::new(Rc::clone(&next_op), &node_id);

            new_inst_vec.push(new_op_value);
            new_inst_hash.insert(next_op.borrow().get_active_base_op().unwrap(), new_inst_vec);

            self.op_manager.insert(key.clone(), new_inst_hash);

            return (true, next_op);
        }

        //let op_node_builder = self.op_manager.get_mut(&key).unwrap().clone();
        //let mut op_node_checker = op_node_builder.clone().to_iter();
        //println!("Previous ops: {:?}", op_node_checker);
        //println!("Checking instruction: {:?}", next_op);

        // This checks all previously added op_nodes to see if any match
        // AND are on the same dominance path.
        // TODO : Attempting to replace this
        /*while let Some(op_node) = op_node_checker.next() {
            println!("Checking op_node: {:?}", op_node);
            if op_node.clone_op() == next_op {
                //println!("Ops are the same!");
                // First check if op is in the same node.
                if op_node.get_node_id() == node_id {
                    return (false, op_node.clone_op());
                }
                // Second, check if it is within the dominance path of nodes.
                if dom_path.contains(&op_node.get_node_id()) {
                    return (false, op_node.clone_op());
                }

                // If neither are true, it is not within the path of dominance,
                // and must be added as a unique instruction.
            }
        }*/

        let mut inst_map = self.op_manager.get(&key).unwrap().clone();
        let contains_op = inst_map.contains_key(&next_op.borrow().get_active_base_op().unwrap());

        /*match contains_op {
            Some((op, op_vec)) => {
                // Start searching for dominating op.
                let node_id_list = op_vec.iter().map(|op_val| {
                    (op_val.clone_node_id(), op_val.clone_op())
                }).collect::<Vec<(NodeIndex, Rc<RefCell<Op>>)>>();

                if let Some(mut dominance_iter) = dom_space.strict_dominators(node_id.clone()) {
                    let

                    while let Some(dominant_node_id) = dominance_iter.next() {

                    }
                }
            },
            None => {
                let mut new_inst_vec = Vec::new();
                let new_op_value = OpValue::new(Rc::clone(&next_op), &node_id);

                new_inst_vec.push(new_op_value);

                self.op_manager.get_mut(&key).unwrap().push((next_op.borrow().get_active_base_op().unwrap(), new_inst_vec));
            }
        }*/

        if !contains_op {
            let mut new_inst_vec = Vec::new();
            let new_op_value = OpValue::new(Rc::clone(&next_op), &node_id);

            new_inst_vec.push(new_op_value);

            self.op_manager.get_mut(&key).unwrap().insert(next_op.borrow().get_active_base_op().unwrap(), new_inst_vec);
        } else {
            let inst_vec = inst_map.get_mut(&next_op.borrow().get_active_base_op().unwrap()).unwrap();

            let search_map = inst_vec
                .iter()
                .map(|value| {
                    (value.clone_node_id(), value.clone_op())
                }).collect::<HashMap<NodeIndex,Rc<RefCell<Op>>>>();

            if search_map.contains_key(&node_id) {
                return (false, Rc::clone(search_map.get(&node_id).unwrap()))
            }

            // Now we make a dominance path, as a very very last resort (should reduce time spent quite a bit)
            let mut dominance_path: Vec<NodeIndex> = Vec::new();
            if let Some(mut dominance_iter) = dom_space.strict_dominators(node_id.clone()) {
                while let Some(dominant_node_id) = dominance_iter.next() {
                    if search_map.contains_key(&dominant_node_id) {
                        return (false, Rc::clone(search_map.get(&dominant_node_id).unwrap()))
                    }
                }
            }

            // If is reaches this point, there are no dominating nodes and it should be added
            let new_op_val = OpValue::new(Rc::clone(&next_op), &node_id);
            self.op_manager.get_mut(&key).unwrap().get_mut(&next_op.borrow().get_active_base_op().unwrap()).unwrap().push(new_op_val);
        }

        (true, next_op)
    }
}

#[derive(Clone, Debug)]
pub struct OpValue {
    op: Rc<RefCell<Op>>,
    node_id: NodeIndex,
}

impl OpValue {
    pub fn new(op: Rc<RefCell<Op>>, node_id: &NodeIndex) -> Self {
        OpValue { op, node_id: node_id.clone() }
    }

    pub fn check_node_id(&self) -> & NodeIndex {
        &self.node_id
    }

    pub fn clone_node_id(&self) -> NodeIndex {
        self.node_id.clone()
    }

    pub fn clone_op(&self) -> Rc<RefCell<Op>> {
        Rc::clone(&self.op)
    }
}

#[derive(Clone, Debug)]
pub struct OpNode {
    op: Rc<RefCell<Op>>,
    node_id: NodeIndex,
    parent_node: Option<Box<OpNode>>,
}

impl OpNode {
    pub fn new_head_node(op_head: Rc<RefCell<Op>>, node_id: &NodeIndex) -> Self {
        OpNode {
            op: op_head.clone(),
            node_id: node_id.clone(),
            parent_node: None,
        }
    }

    pub fn add_op_node(op: Rc<RefCell<Op>>, parent_op: OpNode, node_id: &NodeIndex) -> Self {
        OpNode {
            op: op.clone(),
            node_id: node_id.clone(),
            parent_node: Some(Box::new(parent_op)),
        }
    }

    pub fn get_node_id(&self) -> NodeIndex {
        self.node_id.clone()
    }

    pub fn get_op(&self) -> &Rc<RefCell<Op>> {
        &self.op
    }

    pub fn clone_op(&self) -> Rc<RefCell<Op>> {
        self.op.clone()
    }

    pub fn clone_parent(&self) -> Option<Box<OpNode>> {
        self.parent_node.clone()
    }

    pub fn to_iter(&self) -> OpNodeIterator {
        let parent;
        if let Some(parent_op) = self.clone_parent() {
            parent = Some(*parent_op);
        } else {
            parent = None;
        }

        OpNodeIterator::new(Some(self.clone()), parent)
    }
}

pub struct OpNodeIterator {
    curr: Option<OpNode>,
    next: Option<OpNode>,
}

impl OpNodeIterator {
    pub fn new(curr: Option<OpNode>, next: Option<OpNode>) -> Self {
        OpNodeIterator { curr, next }
    }

    pub fn next(&mut self) -> Option<OpNode> {
        let ret_node = self.curr.clone();
        self.curr = self.next.clone();

        match self.next.clone() {
            Some(op_node) => {
                if let Some(box_node) = op_node.clone_parent() {
                    self.next = Some(*box_node);
                } else {
                    self.next = None;
                }
            }
            None => {
                self.next = None;
            }
        }

        ret_node
    }
}
