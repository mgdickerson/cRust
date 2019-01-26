use lib::IR::ir_manager::IRManager;
use lib::IR::ir::{Op, InstTy};

#[derive(Clone)]
pub struct BasicBlock {
    inst: Vec<Op>,
}

impl BasicBlock {
    pub fn new(irm: &mut IRManager) -> Self {
        irm.inc_block_tracker();
        BasicBlock{ inst: Vec::new() }
    }

    pub fn add_instruction(&mut self, op: Op) {
        self.inst.push(op);
    }

    pub fn to_iter(self) -> std::vec::IntoIter<Op> {
        self.inst.into_iter()
    }

    pub fn get(self) -> Vec<Op> {
        self.inst
    }

    pub fn update(&mut self, instruction_set: Vec<Op>) {
        self.inst = instruction_set;
    }
}

impl std::fmt::Debug for BasicBlock {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        for inst in self.inst.clone() {
            write!(f, "{:?}", inst);
        }

        write!(f, "")
    }
}