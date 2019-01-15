use lib::IR::basic_block::BlockTracker;
use lib::IR::ir::{Value,Op,InstTy,InstTracker};

pub struct IRManager {
    bt: BlockTracker,
    it: InstTracker,
    var_manager: VariableManager,
}

impl IRManager {
    pub fn new() -> Self {
        IRManager { bt: BlockTracker::new(), it: InstTracker::new(), var_manager: VariableManager::new() }
    }

    pub fn build_op(&self, inst_type: InstTy) -> Op {
        Op::build_op(self.get_inst_num(), self.get_block_num(), inst_type)
    }

    pub fn build_op_x(&self, x_val: Value, inst_type: InstTy) -> Op {
        Op::build_op_x(x_val,self.get_inst_num(),self.get_block_num(),inst_type)
    }

    pub fn build_op_x_y(&self, x_val: Value, y_val: Value, inst_type: InstTy) -> Op {
        Op::build_op_x_y(x_val,
                y_val,
                self.get_inst_num(),
                self.get_block_num(),
                inst_type)
    }

    pub fn build_op_y(&self, y_val: Value, inst_type: InstTy) -> Op {
        Op::build_op_y(y_val, self.get_inst_num(), self.get_block_num(), inst_type)
    }

    pub fn build_spec_op(&self, special_val: Vec<Box<Value>>, inst_type: InstTy) -> Op {
        Op::build_spec_op(special_val,self.get_inst_num(),self.get_block_num(),inst_type)
    }

    pub fn inc_inst_tracker(&mut self) {
        self.it.increment();
    }

    pub fn inc_block_tracker(&mut self) {
        self.bt.increment();
    }

    pub fn get_inst_num(&self) -> usize {
        self.it.get()
    }

    pub fn get_block_num(&self) -> usize {
        self.bt.get()
    }
}

pub struct VariableManager {

}

impl VariableManager {
    pub fn new() -> Self {

    }
}

pub struct UniqueVariable {
    unique_ident: String,
    def: String,
    used: Vec<String>,
}

impl UniqueVariable {
    pub fn new(ident: String, count: usize, def: String) -> Self {
        
    }
}