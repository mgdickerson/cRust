use lib::IR::ir_manager::InstTracker;
use lib::IR::ir::{Op, InstTy};

use super::{Rc,RefCell};

#[derive(Clone)]
pub struct BasicBlock {
    inst: Vec<Rc<RefCell<Op>>>,
}

impl BasicBlock {
    pub fn new(it: &mut InstTracker) -> Self {
        it.increment();
        BasicBlock{ inst: Vec::new() }
    }

    pub fn add_instruction(&mut self, op: Rc<RefCell<Op>>) {
        self.inst.push(op);
    }

    pub fn insert_instruction(&mut self, position: usize, op: Rc<RefCell<Op>>) {
        self.inst.insert(position, op);
    }

    pub fn to_iter(self) -> std::vec::IntoIter<Rc<RefCell<Op>>> {
        self.inst.into_iter()
    }

    pub fn get(self) -> Vec<Rc<RefCell<Op>>> {
        self.inst
    }

    pub fn get_inst_list_ref(&mut self) -> &mut Vec<Rc<RefCell<Op>>> {
        &mut self.inst
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