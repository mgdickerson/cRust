use super::{Rc,RefCell,HashMap};
use super::{Value,ValTy, Op};
use std::cell::Ref;
use lib::Graph::graph_manager::GraphManager;
use petgraph::prelude::NodeIndex;
use petgraph::visit::DfsPostOrder;
use std::fmt::Debug;

#[derive(Clone)]
pub struct TempValManager {
    temp_vec: Vec<Rc<RefCell<TempVal>>>,
    op_hash: HashMap<usize,Rc<RefCell<TempVal>>>,
    graph_visit_order: Vec<NodeIndex>,
}

impl TempValManager {
    pub fn new() -> Self {
        TempValManager {
            temp_vec: Vec::new(),
            op_hash: HashMap::new(),
            graph_visit_order: Vec::new(),
        }
    }

    pub fn clone_visit_order(&self) -> Vec<NodeIndex> {
        self.graph_visit_order.clone()
    }

    pub fn pull_temp_values(&mut self, graph_manager: &GraphManager, entry_node: NodeIndex) {
        let mut dfs_post_order = DfsPostOrder::new(graph_manager.get_ref_graph(), entry_node);
        let mut graph_visitor = Vec::new();
        let mut revisit_inst = Vec::new();

        // This gets a pretty good order of nodes to visit, though for some reason it still goes down the right path first.
        while let Some(node_id) = dfs_post_order.next(graph_manager.get_ref_graph()) {
            graph_visitor.push(node_id);
        }

        graph_visitor.reverse();
        self.graph_visit_order = graph_visitor.clone();
        //println!("{:?}", graph_visitor);

        // Will have to make a second loop for the Phi statements of whiles (because the Phi instructions will not have access to all
        // instructions as some will be called later in the cycle). Thus a clean up second cycle will be needed to finish adding
        // all uses.
        for node in graph_visitor.iter() {
            // iterate through instructions in each node
            for inst in graph_manager.get_ref_graph().node_weight(node.clone()).unwrap().get_data_ref().get_inst_list_ref() {
                self.add_inst(inst, &mut revisit_inst);
            }
        }

        // Now revisit instruction that used values before they were added to the original map
        for (node_id, temp_val) in revisit_inst.iter() {
            self.op_hash.get_mut(&node_id).expect("While adding temp values, clean up routine found value not already added.")
                .borrow_mut().add_use(Rc::clone(temp_val));
        }

        // Testing equipment below.
        //println!("{:?}", revisit_inst);

        /*let mut sorted_op_hash = self.op_hash.iter()
            .map(|(key,val)| {
                (key, val)
            }).collect::<Vec<_>>();
        sorted_op_hash.sort_by_key(|(key,val)| key.clone().clone());

        // Quick test of op_hash values
        for (inst_num, inst) in sorted_op_hash.iter() {
            println!("[{}]: Uses -> {:?}", inst_num, inst.borrow().active_uses());
        }*/
    }

    pub fn add_inst(&mut self, inst: &Rc<RefCell<Op>>, inst_revisit_vec: &mut Vec<(usize, Rc<RefCell<TempVal>>)>) {
        // Make a new TempVal using the passed inst
        let inst_num = inst.borrow().get_inst_num();
        let new_temp = TempVal::new(inst, inst_num);

        // Make Rc<RefCell<_>> out of TempVal
        let ref_temp = Rc::new(RefCell::new(new_temp.clone()));

        // Check to see if this value calls any other previously added values
        if let Some(x_val) = new_temp.x_val() {
            if let ValTy::op(op) = x_val.get_value() {
                let temp_inst_num = op.borrow().get_inst_num();
                if self.op_hash.contains_key(&temp_inst_num) {
                    // x_val is already added to hash_map, add use
                    self.op_hash.get_mut(&temp_inst_num).unwrap().borrow_mut().add_use(Rc::clone(&ref_temp));
                } else {
                    // x_val is not already part of map, add to revisit list
                    inst_revisit_vec.push((temp_inst_num, Rc::clone(&ref_temp)));
                }
            }
        }

        if let Some(y_val) = new_temp.y_val() {
            if let ValTy::op(op) = y_val.get_value() {
                let temp_inst_num = op.borrow().get_inst_num();
                if self.op_hash.contains_key(&temp_inst_num) {
                    // y_val is already added to hash_map, add use
                    self.op_hash.get_mut(&temp_inst_num).unwrap().borrow_mut().add_use(Rc::clone(&ref_temp));
                } else {
                    // y_val is not already part of map, add to revisit list
                    inst_revisit_vec.push((temp_inst_num, Rc::clone(&ref_temp)));
                }
            }
        }

        // After adding uses, add this instruction's temp_val to the op_hash
        self.op_hash.insert(inst_num, Rc::clone(&ref_temp));

        // Also add to the temp_vec
        self.temp_vec.push(Rc::clone(&ref_temp));
    }
}

#[derive(Clone)]
pub struct TempVal {
    // Value of the temp value
    op_val: Rc<RefCell<Op>>,

    // where instruction is created
    block_num: usize,
    inst_num: usize,

    // where value is used
    used: Vec<Rc<RefCell<TempVal>>>,

    // operands
    x_val: Option<Value>,
    y_val: Option<Value>,

    // is value still active in graph
    is_active: bool,
}

impl TempVal {
    pub fn new(inst: &Rc<RefCell<Op>>, inst_num: usize) -> Self {
        let (x_val, y_val, _) = inst.borrow().get_values();
        let block_num = inst.borrow().get_inst_block();
        TempVal {
            op_val: Rc::clone(inst),
            block_num,
            inst_num,
            used: Vec::new(),
            x_val,
            y_val,
            is_active: true,
        }
    }

    pub fn inst_val(&self) -> Rc<RefCell<Op>> {
        Rc::clone(&self.op_val)
    }

    pub fn x_val(&self) -> Option<Value> {
        self.x_val.clone()
    }

    pub fn y_val(&self) -> Option<Value> {
        self.y_val.clone()
    }

    pub fn block_num(&self) -> usize {
        self.block_num.clone()
    }

    pub fn inst_num(&self) -> usize {
        self.inst_num.clone()
    }

    pub fn add_use(&mut self, temp_val_clone: Rc<RefCell<TempVal>>) {
        self.used.push(temp_val_clone);
    }

    pub fn is_used(&self) -> bool {
        !self.used.is_empty()
    }

    pub fn active_uses(&self) -> Vec<Rc<RefCell<TempVal>>> {
        self.used.iter().filter(|temp_val| {
            temp_val.borrow().is_active()
        }).map(|temp_val| {
            Rc::clone(temp_val)
        }).collect::<Vec<_>>()
    }

    pub fn is_active(&self) -> bool {
        self.is_active.clone()
    }
}

impl std::fmt::Debug for TempVal {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self.op_val)
    }
}