use super::{HashMap, Rc, RefCell};
use super::{Op, ValTy, Value};
use lib::Graph::graph_manager::GraphManager;
use lib::Optimizer::Optimizer;
use lib::IR::ir::InstTy;
use petgraph::prelude::NodeIndex;
use petgraph::visit::DfsPostOrder;
use std::cell::Ref;
use std::fmt::Debug;

#[derive(Clone)]
pub struct TempValManager {
    temp_vec: Vec<Rc<RefCell<TempVal>>>,
    op_hash: HashMap<usize, Rc<RefCell<TempVal>>>,
}

impl TempValManager {
    pub fn new() -> Self {
        TempValManager {
            temp_vec: Vec::new(),
            op_hash: HashMap::new(),
        }
    }

    pub fn pull_temp_values(&mut self, graph_manager: &GraphManager, entry_node: NodeIndex) {
        let graph_visitor = graph_manager.graph_visitor(entry_node);

        let mut revisit_inst = Vec::new();

        // Will have to make a second loop for the Phi statements of whiles (because the Phi instructions will not have access to all
        // instructions as some will be called later in the cycle). Thus a clean up second cycle will be needed to finish adding
        // all uses.
        for node in graph_visitor.iter() {
            // iterate through instructions in each node
            for inst in graph_manager
                .get_ref_graph()
                .node_weight(node.clone())
                .unwrap()
                .get_data_ref()
                .get_inst_list_ref()
            {
                self.add_inst(inst, &mut revisit_inst);
            }
        }

        // Now revisit instruction that used values before they were added to the original map
        for (node_id, temp_val) in revisit_inst.iter() {
            self.op_hash
                .get_mut(&node_id)
                .expect("While adding temp values, clean up routine found value not already added.")
                .borrow_mut()
                .add_use(Rc::clone(temp_val));
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

    pub fn add_inst(
        &mut self,
        inst: &Rc<RefCell<Op>>,
        inst_revisit_vec: &mut Vec<(usize, Rc<RefCell<TempVal>>)>,
    ) {
        // Make a new TempVal using the passed inst
        let inst_num = inst.borrow().get_inst_num();
        let mut new_temp = TempVal::new(inst, inst_num);

        // Make Rc<RefCell<_>> out of TempVal
        let ref_temp = Rc::new(RefCell::new(new_temp.clone()));

        let static_temp = Rc::new(RefCell::new(new_temp.clone()));

        let inst_ty = inst.borrow().inst_type().clone();
        match inst_ty {
            InstTy::read
            | InstTy::add
            | InstTy::sub
            | InstTy::mul
            | InstTy::div
            | InstTy::cmp
            | InstTy::adda
            | InstTy::phi
            | InstTy::load => {
                // TODO : Should this be removed and handled in dead code elimination?
                // It is probably better practice to remove the dead code based on
                // having no active uses in a separate sweep. Code that should not
                // be removed will be protected by having an always constant use of
                // itself.
                //inst.borrow_mut().deactivate();
            }
            _ => {
                ref_temp.borrow_mut().add_use(static_temp);
            }
        }

        // Check to see if this value calls any other previously added values
        if let Some(x_val) = new_temp.x_val() {
            if let ValTy::op(op) = x_val.get_value() {
                let temp_inst_num = op.borrow().get_inst_num();
                if self.op_hash.contains_key(&temp_inst_num) {
                    // x_val is already added to hash_map, add use
                    self.op_hash
                        .get_mut(&temp_inst_num)
                        .unwrap()
                        .borrow_mut()
                        .add_use(Rc::clone(&ref_temp));
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
                    self.op_hash
                        .get_mut(&temp_inst_num)
                        .unwrap()
                        .borrow_mut()
                        .add_use(Rc::clone(&ref_temp));
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

    pub fn borrow_mut_inst(&mut self, inst_id: &usize) -> &mut Rc<RefCell<TempVal>> {
        self.op_hash
            .get_mut(inst_id)
            .expect("Attempted to mutably borrow non-existent instruction.")
    }

    pub fn borrow_inst(&self, inst_id: &usize) -> &Rc<RefCell<TempVal>> {
        self.op_hash.get(inst_id).expect(
            &format!(
                "Attempted to borrow non-existent instruction. {:?}",
                inst_id
            )[..],
        )
    }

    pub fn update_inst_uses(&self, inst_id: &usize, new_val: Value) {
        self.op_hash
            .get(&inst_id)
            .unwrap()
            .borrow()
            .update_use_values(new_val);
    }

    pub fn update_inst_x_val(&mut self, inst_id: &usize, x_val: Value) {
        self.op_hash.get_mut(inst_id).expect(&format!(
            "Attempted to alter x_val of non-existent instruction. {:?}",
            inst_id
            )[..])
            .borrow_mut()
            .update_x_val(x_val);
    }

    pub fn update_inst_y_val(&mut self, inst_id: &usize, y_val: Value) {
        self.op_hash.get_mut(inst_id).expect(&format!(
            "Attempted to alter y_val non-existent instruction. {:?}",
            inst_id
        )[..])
            .borrow_mut()
            .update_y_val(y_val);
    }

    pub fn get_inactive_list(&self) -> Vec<&Rc<RefCell<TempVal>>> {
        self.op_hash
            .values()
            .filter(|value| !value.borrow().is_active())
            .collect::<Vec<_>>()
    }

    pub fn check_active_values(&self, inst_id: &usize) -> (bool, bool) {
        let temp_val = self
            .op_hash
            .get(inst_id)
            .expect("Values attempting to be checked should be valid");
        let mut x_valid = false;
        let mut y_valid = false;

        if let Some(x_value) = temp_val.borrow().x_val() {
            match x_value.clone_value() {
                ValTy::op(x_op) => {
                    //println!("x_op: {:?} is_active: {}", x_op, x_op.borrow().is_active());
                    x_valid = x_op.borrow().is_active();
                }
                _ => {
                    x_valid = true;
                }
            }
        }

        if let Some(y_value) = temp_val.borrow().y_val() {
            match y_value.clone_value() {
                ValTy::op(y_op) => {
                    //println!("x_op: {:?} is_active: {}", y_op, y_op.borrow().is_active());
                    y_valid = y_op.borrow().is_active();
                }
                _ => {
                    y_valid = true;
                }
            }
        }

        (x_valid, y_valid)
    }

    pub fn clean_instruction_uses(&mut self, inst_id: &usize) {
        //println!("Are we panicing here?");

        let mut local_handle = self.op_hash.get_mut(inst_id).unwrap().clone();

        //println!("Are we panicing here2?");
        let x_val = local_handle.borrow().x_val();
        if let ValTy::op(x_op) = x_val.unwrap().get_value().clone() {
            let x_inst_id = x_op.borrow().get_inst_num();
            self.op_hash
                .get_mut(&x_inst_id)
                .unwrap()
                .borrow_mut()
                .remove_use(inst_id);
        }

        //println!("Are we panicing here3?");
        let y_val = local_handle.borrow().y_val();
        if let ValTy::op(y_op) = y_val.unwrap().get_value().clone() {
            let y_inst_id = y_op.borrow().get_inst_num();
            self.op_hash
                .get_mut(&y_inst_id)
                .unwrap()
                .borrow_mut()
                .remove_use(inst_id);
        }
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
    used: HashMap<usize, Rc<RefCell<TempVal>>>,
}

impl TempVal {
    pub fn new(inst: &Rc<RefCell<Op>>, inst_num: usize) -> Self {
        let block_num = inst.borrow().get_inst_block();
        TempVal {
            op_val: Rc::clone(inst),
            block_num,
            inst_num,
            used: HashMap::new(),
        }
    }

    pub fn inst_val(&self) -> Rc<RefCell<Op>> {
        Rc::clone(&self.op_val)
    }

    pub fn inst_type(&self) -> InstTy {
        self.op_val.borrow().inst_type().clone()
    }

    pub fn x_y_val(&self) -> (Option<Value>, Option<Value>) {
        (
            self.op_val.borrow().clone_x_val(),
            self.op_val.borrow().clone_y_val(),
        )
    }

    pub fn x_val(&self) -> Option<Value> {
        self.op_val.borrow().clone_x_val()
    }

    pub fn update_x_val(&mut self, x_val: Value) {
        self.op_val.borrow_mut().update_x_val(x_val);
    }

    pub fn y_val(&self) -> Option<Value> {
        self.op_val.borrow().clone_y_val()
    }

    pub fn update_y_val(&mut self, y_val: Value) {
        self.op_val.borrow_mut().update_y_val(y_val);
    }

    pub fn block_num(&self) -> usize {
        self.block_num.clone()
    }

    pub fn inst_num(&self) -> usize {
        self.inst_num.clone()
    }

    pub fn update_use_values(&self, new_val: Value) {
        let old_val = Value::new(ValTy::op(self.op_val.clone()));
        for active_use in self.active_uses().iter() {
            let use_op = active_use.borrow().inst_val();
            println!("Failure on active use: {}", use_op.borrow().get_inst_num());
            use_op
                .borrow_mut()
                .var_cleanup(old_val.clone(), new_val.clone());
        }
    }

    pub fn add_use(&mut self, temp_val_clone: Rc<RefCell<TempVal>>) {
        self.op_val.borrow_mut().activate();
        let temp_id = temp_val_clone.borrow().inst_num();
        self.op_val.borrow_mut().activate();
        self.used.insert(temp_id, temp_val_clone);
    }

    pub fn is_used(&self) -> bool {
        !self.used.is_empty()
    }

    pub fn active_uses(&self) -> Vec<Rc<RefCell<TempVal>>> {
        self.used
            .values()
            .filter(|temp_val| temp_val.borrow().is_active())
            .map(|temp_val| Rc::clone(temp_val))
            .collect::<Vec<_>>()
    }

    pub fn remove_use(&mut self, remove_id: &usize) -> bool {
        if let Some(value) = self.used.remove(remove_id) {
            // After successfully removing a use from this temp_val,
            // if it was the last use mark it as no longer active.
            let active_uses = self
                .used
                .values()
                .filter(|temp_val| temp_val.borrow().is_active())
                .collect::<Vec<_>>();

            if active_uses.is_empty() {
                self.op_val.borrow_mut().deactivate();
            }

            // Removed Valid use
            true
        } else {
            // Attempted to remove invalid use
            false
        }
    }

    pub fn deactivate_instruction(&mut self) {
        self.op_val.borrow_mut().deactivate();
    }

    pub fn is_active(&self) -> bool {
        self.op_val.borrow().is_active()
    }
}

impl std::fmt::Debug for TempVal {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self.op_val)
    }
}
