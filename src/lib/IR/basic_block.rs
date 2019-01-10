use lib::IR::ir::Instruction;

#[derive(Debug,Clone)]
pub struct BasicBlock {
    inst: Vec<TyPe>,
}

impl BasicBlock {
    pub fn new() -> Self {
        BasicBlock{ inst: Vec::new() }
    }
}

/// Temp Type
#[derive(Debug,Clone)]
pub enum TyPe {
    placeHolder,
}