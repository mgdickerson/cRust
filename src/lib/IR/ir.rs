use lib::IR::address_manager::UniqueAddress;
use lib::IR::array_manager::UniqueArray;
use lib::IR::ret_register::RetRegister;
use lib::IR::variable_manager::{UniqueVariable, VariableManager};

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use super::{Rc, RefCell};
use lib::RegisterAllocator::RegisterAllocation;
use petgraph::graph::NodeIndex;

#[derive(Debug, Clone, PartialEq, Hash)]
pub struct Value {
    val: ValTy,
}

impl Value {
    pub fn new(val_ty: ValTy) -> Self {
        Value { val: val_ty }
    }

    pub fn get_value(&self) -> &ValTy {
        &self.val
    }

    pub fn update_value(&mut self, new_val_ty: ValTy) {
        self.val = new_val_ty;
    }

    pub fn get_var_base(&self) -> ValTy {
        let ret;
        if let ValTy::var(var) = self.clone_value() {
            ret = var.borrow().get_value().to_owned().clone().get_var_base()
        } else {
            ret = self.clone_value()
        }

        ret
    }

    pub fn clone_value(&self) -> ValTy {
        self.val.clone()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ValTy {
    op(Rc<RefCell<Op>>),
    node_id(NodeIndex),
    con(i32),
    var(Rc<RefCell<UniqueVariable>>),
    adr(UniqueAddress),
    arr(UniqueArray),
    ret(RetRegister),
    reg(RegisterAllocation),
}

impl Hash for ValTy {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match &self {
            ValTy::op(op) => {
                op.borrow().get_inst_num().hash(state);
            }
            ValTy::node_id(node_id) => {
                node_id.index().hash(state);
            },
            ValTy::con(con) => {
                con.clone().hash(state);
            },
            ValTy::var(var) => {
                0.hash(state);
            },
            ValTy::adr(adr) => {
                adr.to_string().hash(state);
            },
            ValTy::arr(arr) => {
                arr.to_string().hash(state);
            },
            ValTy::ret(ret) => {
                ret.to_string().hash(state);
            },
            ValTy::reg(reg) => {
                reg.to_string().hash(state);
            },
        }
    }
}

impl ValTy {
    pub fn to_string(&self) -> String {
        match &self {
            ValTy::op(op) => op.borrow().get_return_value(),
            ValTy::node_id(id) => String::from("[") + &id.index().to_string() + "]",
            ValTy::con(con) => String::from("#") + &con.to_string(),
            ValTy::var(var) => {
                // Temporarily, I want it to output var name
                var.borrow().value_to_string()
                //var.borrow().get_ident()
            }
            ValTy::adr(adr) => adr.to_string(),
            ValTy::arr(arr) => arr.to_string(),
            ValTy::ret(ret) => ret.to_string(),
            ValTy::reg(reg) => reg.to_string(),
        }
    }
}

#[derive(Clone)]
pub struct Op {
    // Value Operands
    x_val: Option<Value>,
    y_val: Option<Value>,
    special_val: Option<String>,

    // General Information about self
    inst_number: usize,
    block_number: usize,
    inst_type: InstTy,
    is_active: bool,

    // Useful for debugging or printing
    p_command: String,
}

impl Op {
    pub fn new(
        x_val: Option<Value>,
        y_val: Option<Value>,
        special_val: Option<String>,
        inst_number: usize,
        block_number: usize,
        inst_type: InstTy,
    ) -> Self {
        let mut p_command = String::new();

        Op {
            x_val,
            y_val,
            special_val,
            inst_number,
            block_number,
            inst_type,
            is_active: true,
            p_command,
        }
    }

    pub fn build_op(
        x_val: Option<Value>,
        y_val: Option<Value>,
        special_val: Option<String>,
        block_number: usize,
        inst_number: usize,
        inst_type: InstTy,
        var_manager: &mut VariableManager,
    ) -> Op {
        // TODO : now I can add uses to variables as the operation is being built. should make the numbers far more accurate.
        match &x_val {
            Some(val) => match val.get_value() {
                ValTy::var(var) => {
                    var.borrow_mut().add_use(block_number, inst_number);
                }
                _ => {}
            },
            None => {}
        }

        match &y_val {
            Some(val) => match val.get_value() {
                ValTy::var(var) => {
                    var.borrow_mut().add_use(block_number, inst_number);
                }
                _ => {}
            },
            None => {}
        }

        Op::new(
            x_val,
            y_val,
            special_val,
            inst_number,
            block_number,
            inst_type,
        )
    }

    // TODO : Switch string building out of initialization and make it part of this command.
    pub fn to_string(&self) -> String {
        let mut p_command = String::new();
        let inst_type = self.inst_type.clone();

        match &inst_type.clone() {
            // Op //
            InstTy::read | InstTy::end | InstTy::writeNL => {
                p_command = inst_type.to_string();
            }
            // Op x //
            InstTy::neg | InstTy::write | InstTy::ret => {
                p_command = inst_type.to_string()
                    + " "
                    + &self.x_val.clone().unwrap().get_value().to_string();
            }
            // Op x y //
            InstTy::add
            | InstTy::sub
            | InstTy::mul
            | InstTy::div
            | InstTy::cmp
            | InstTy::adda
            | InstTy::bne
            | InstTy::beq
            | InstTy::ble
            | InstTy::blt
            | InstTy::bge
            | InstTy::bgt
            | InstTy::phi => {
                p_command = inst_type.to_string()
                    + " "
                    + &self.x_val.clone().unwrap().get_value().to_string()
                    + " "
                    + &self.y_val.clone().unwrap().get_value().to_string();
            }
            // Op y //
            InstTy::load | InstTy::bra => {
                p_command = inst_type.to_string()
                    + " "
                    + &self.y_val.clone().unwrap().get_value().to_string();
            }
            // Op y x //
            InstTy::store | InstTy::mov => {
                p_command = inst_type.to_string()
                    + " "
                    + &self.y_val.clone().unwrap().get_value().to_string()
                    + " "
                    + &self.x_val.clone().unwrap().get_value().to_string();
            }
            // Op [x] //
            InstTy::call => {
                p_command = String::from("call ");
                match &self.special_val {
                    Some(val_str) => {
                        p_command += &val_str;
                    }
                    None => {
                        panic!("Should probably always have a string value.");
                    }
                }
            }

            _ => {
                panic!("Error in Op construction, unexpected inst_type found.");
            }
        }

        p_command
    }

    pub fn get_active_base_op(&self) -> Option<Op> {
        if !self.is_active {
            None
        } else {
            Some(self.clone())
        }
    }

    pub fn update_inst_ty(&mut self, new_inst_ty: InstTy) {
        self.inst_type = new_inst_ty;
    }

    pub fn is_active(&self) -> bool {
        self.is_active.clone()
    }

    pub fn activate(&mut self) {
        self.is_active = true;
    }

    pub fn deactivate(&mut self) {
        self.is_active = false;
    }

    pub fn get_values(&self) -> (Option<Value>, Option<Value>, Option<String>) {
        (
            self.x_val.clone(),
            self.y_val.clone(),
            self.special_val.clone(),
        )
    }

    pub fn get_val_ty(&self) -> (Option<ValTy>, Option<ValTy>) {
        let x_val;
        let y_val;

        match &self.x_val {
            Some(x_value) => {
                x_val = Some(x_value.get_value().clone());
            }
            None => {
                x_val = None;
            }
        }

        match &self.y_val {
            Some(y_value) => {
                y_val = Some(y_value.get_value().clone());
            }
            None => {
                y_val = None;
            }
        }

        (x_val, y_val)
    }

    pub fn clone_x_val(&self) -> Option<Value> {
        self.x_val.clone()
    }

    pub fn update_x_val(&mut self, new_val: Value) {
        self.x_val = Some(new_val);
    }

    pub fn clone_y_val(&self) -> Option<Value> {
        self.y_val.clone()
    }

    pub fn update_y_val(&mut self, new_val: Value) {
        self.y_val = Some(new_val);
    }

    pub fn update_special_val(&mut self, new_val: String) {
        self.special_val = Some(new_val);
    }

    /// Grabs the base value of variables and sets that as the new value of x or y val
    pub fn update_base_values(&mut self) {
        if let Some(x_val) = self.x_val.clone() {
            //println!("Value x is currently: {:?}", self.x_val.clone().unwrap());
            let val_ty = x_val.get_var_base();
            self.x_val = Some(Value::new(val_ty));
            //println!("Adding value {:?} to x", self.x_val.clone().unwrap())
        }

        if let Some(y_val) = self.y_val.clone() {
            //println!("Value y is currently: {:?}", self.y_val.clone().unwrap());
            let val_ty = y_val.get_var_base();
            self.y_val = Some(Value::new(val_ty));
            //println!("Adding value {:?} to y", self.y_val.clone().unwrap())
        }
    }

    pub fn get_return_value(&self) -> String {
        let string = String::from("(") + &self.inst_number.to_string() + ")";
        string
    }

    pub fn get_inst_block(&self) -> usize {
        self.block_number.clone()
    }

    pub fn get_inst_num(&self) -> usize {
        self.inst_number.clone()
    }

    pub fn update_inst_num(&mut self, new_inst_num: &usize) {
        self.inst_number = new_inst_num.clone();
    }

    pub fn inst_type(&self) -> &InstTy {
        &self.inst_type
    }

    pub fn op_cleanup(&mut self, var_to_clean: usize, replacement_op: Value) {
        match self.x_val.clone() {
            Some(val) => {
                if let ValTy::op(op) = val.clone_value() {
                    let op_id = op.borrow().get_inst_num();
                    if op_id == var_to_clean {
                        self.x_val = Some(replacement_op.clone());
                    }
                }
            }
            None => {}
        }

        match self.y_val.clone() {
            Some(val) => {
                if let ValTy::op(op) = val.clone_value() {
                    let op_id = op.borrow().get_inst_num();
                    if op_id == var_to_clean {
                        self.y_val = Some(replacement_op.clone());
                    }
                }
            }
            None => {}
        }
    }

    pub fn var_cleanup(&mut self, var_to_clean: Value, replacement_var: Value) {
        //println!("x_val: {:?}, to_clean_val: {:?}", self.x_val.clone(), var_to_clean.clone());
        match self.x_val.clone() {
            Some(val) => {
                if val == var_to_clean {
                    //println!("Clean cycle reaches x_val replacement for {}.", var_to_clean.get_value().to_string());
                    self.x_val = Some(replacement_var.clone());
                }
            }
            None => {
                // There is no variable to clean, pass through.
            }
        }

        //println!("y_val: {:?}, to_clean_val: {:?}", self.y_val.clone(), var_to_clean.clone());
        match self.y_val.clone() {
            Some(val) => {
                if val == var_to_clean {
                    //println!("Clean cycle reaches y_val replacement for {}.", var_to_clean.get_value().to_string());
                    self.y_val = Some(replacement_var.clone());
                }
            }
            None => {
                // There is no variable to clean, pass through.
            }
        }
    }
}

impl std::fmt::Debug for Op {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "({}): {}; \\l ", self.inst_number, self.to_string())
    }
}

impl Hash for Op {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.inst_type.hash(state);
        match &self.x_val {
            Some(x_val) => {
                x_val.clone().hash(state);
            },
            None => {
                0.hash(state);
            }
        }
        match &self.y_val {
            Some(y_val) => {
                y_val.clone().hash(state);
            },
            None => {
                0.hash(state);
            }
        }
    }
}

impl Eq for Op {}

impl PartialEq for Op {
    fn eq(&self, other: &Op) -> bool {
        if self.inst_type == other.inst_type {
            let (self_x, self_y) = self.get_val_ty();
            let (other_x, other_y) = other.get_val_ty();

            match (self_x, other_x) {
                (Some(self_some_x), Some(other_some_x)) => {
                    if let ValTy::op(self_op_x) = self_some_x {
                        let x_inst_num = self_op_x.borrow().get_inst_num();
                        //println!("Passes first op_check: {}", x_inst_num);
                        if let ValTy::op(other_op_x) = other_some_x {
                            //println!("Passes second op check");
                            let x_other_inst_num = other_op_x.borrow().get_inst_num();
                            //println!("{} == {} ?", x_inst_num, x_other_inst_num);
                            if x_inst_num == x_other_inst_num {
                                //println!("X_inst is same as Other_x_inst");
                                match (self_y, other_y) {
                                    (Some(self_some_y), Some(other_some_y)) => {
                                        if let ValTy::op(self_op_y) = self_some_y {
                                            let y_inst_num = self_op_y.borrow().get_inst_num();
                                            if let ValTy::op(other_op_y) = other_some_y {
                                                let y_other_inst_num =
                                                    other_op_y.borrow().get_inst_num();
                                                if y_inst_num == y_other_inst_num {
                                                    return true;
                                                } else {
                                                    return false;
                                                }
                                            } else {
                                                return false;
                                            }
                                        }
                                    }
                                    (None, None) => {
                                        return true;
                                    }
                                    _ => {
                                        return false;
                                    }
                                }
                            } else {
                                return false;
                            }
                        } else {
                            return false;
                        }
                    }
                }
                (None, None) => {
                    return true;
                }
                _ => {
                    return false;
                }
            }

            if self.x_val == other.x_val {
                if self.y_val == other.y_val {
                    if self.special_val == other.special_val {
                        return true;
                    }
                }
            }
        }

        false
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum InstTy {
    /// Op ///
    read,
    end,
    writeNL,

    /// Op x ///
    neg,
    write,
    ret,

    /// Op x y ///
    add,
    sub,
    mul,
    div,
    cmp,
    adda,

    bne,
    beq,
    ble,
    blt,
    bge,
    bgt,

    phi,

    /// Op y ///
    load,
    bra,

    /// Op y x ///
    store,
    mov,

    /// Op Str ///
    call,
}

impl InstTy {
    pub fn to_string(&self) -> String {
        match self {
            /// Op ///
            InstTy::read => String::from("read"),
            InstTy::end => String::from("end"),
            InstTy::writeNL => String::from("writeNL"),

            /// Op x ///
            InstTy::neg => String::from("neg"),
            InstTy::write => String::from("write"),
            InstTy::ret => String::from("ret"),

            /// Op x y ///
            InstTy::add => String::from("add"),
            InstTy::sub => String::from("sub"),
            InstTy::mul => String::from("mul"),
            InstTy::div => String::from("div"),
            InstTy::cmp => String::from("cmp"),
            InstTy::adda => String::from("adda"),

            InstTy::bne => String::from("bne"),
            InstTy::beq => String::from("beq"),
            InstTy::ble => String::from("ble"),
            InstTy::blt => String::from("blt"),
            InstTy::bge => String::from("bge"),
            InstTy::bgt => String::from("bgt"),

            InstTy::phi => String::from("phi"),

            /// Op y ///
            InstTy::load => String::from("load"),
            InstTy::bra => String::from("bra"),

            /// Op y x ///
            InstTy::store => String::from("store"),
            InstTy::mov => String::from("move"),

            /// Op [x] ///
            InstTy::call => String::from("call"),

            _ => {
                panic!("Error occurred, was not a default type.");
            }
        }
    }
}
