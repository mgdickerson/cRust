use lib::IR::ir::Inst;

#[derive(Clone)]
pub struct BasicBlock {
    inst: Vec<Inst>,
}

impl BasicBlock {
    pub fn new() -> Self {
        BasicBlock{ inst: Vec::new() }
    }

    pub fn add_instruction(&mut self, inst: Inst) {
        self.inst.push(inst);
    }

    pub fn to_iter(self) -> std::vec::IntoIter<Inst> {
        self.inst.into_iter()
    }

    pub fn get(self) -> Vec<Inst> {
        self.inst
    }

    pub fn update(&mut self, instruction_set: Vec<Inst>) {
        self.inst = instruction_set;
    }
}

impl std::fmt::Debug for BasicBlock {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        for inst in self.inst.clone() {
            write!(f, "{:?}", inst.get_inst().unwrap());
        }

        write!(f, "")
    }
}