use lib::IR::ir::{InstTy, Op};
use lib::IR::ir_manager::InstTracker;

use super::{Rc, RefCell};

#[derive(Clone)]
pub struct BasicBlock {
    inst: Vec<Rc<RefCell<Op>>>,
}

impl BasicBlock {
    pub fn new() -> Self {
        //it.increment();
        BasicBlock { inst: Vec::new() }
    }

    pub fn add_instruction(&mut self, op: Rc<RefCell<Op>>) {
        self.inst.push(op);
    }

    pub fn insert_instruction(&mut self, position: usize, op: Rc<RefCell<Op>>) {
        self.inst.insert(position, op);
    }

    pub fn remove_inactive_inst(&mut self) {
        // Iterate through list of instructions checking if they are still active
        let active_inst = self
            .inst
            .iter()
            .filter(|op| op.borrow().is_active())
            .map(|filtered_ops| Rc::clone(filtered_ops))
            .collect::<Vec<_>>();

        // Replace instruction vector with active_inst vector
        self.inst = active_inst;
    }

    pub fn to_iter(self) -> std::vec::IntoIter<Rc<RefCell<Op>>> {
        self.inst.into_iter()
    }

    pub fn get(self) -> Vec<Rc<RefCell<Op>>> {
        self.inst
    }

    pub fn get_mut_inst_list_ref(&mut self) -> &mut Vec<Rc<RefCell<Op>>> {
        &mut self.inst
    }

    pub fn get_inst_list_ref(&self) -> &Vec<Rc<RefCell<Op>>> {
        &self.inst
    }

    pub fn update(&mut self, instruction_set: Vec<Rc<RefCell<Op>>>) {
        self.inst = instruction_set;
    }
}

impl std::fmt::Debug for BasicBlock {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        for inst in self.inst.clone() {
            write!(f, "{:?}", inst.borrow());
        }

        write!(f, "")
    }
}
