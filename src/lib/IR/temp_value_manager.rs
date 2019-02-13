use super::{Rc,RefCell};
use lib::IR::ir::Op;

// This will be the new handler for inst_number by taking size of vec (will start at 0)
pub struct TempValManager {
    temp_vec: Vec<Rc<RefCell<Op>>>,
}

impl TempValManager {
    pub fn new() -> Self {
        TempValManager {
            temp_vec: Vec::new(),
        }
    }

    pub fn add_inst(&mut self, inst: Op) -> Rc<RefCell<Op>> {
        // Create Rc ref value and add to the temp_vec
        let temp = Rc::new(RefCell::new(inst));
        self.temp_vec.push(Rc::clone(&temp));

        // Create another reference to this and send back
        Rc::clone(&temp)
    }
}