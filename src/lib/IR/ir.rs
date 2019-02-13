use lib::IR::variable_manager::{UniqueVariable, VariableManager};
use lib::IR::array_manager::UniqueArray;
use lib::IR::address_manager::UniqueAddress;
use lib::IR::ret_register::RetRegister;

use super::{Rc,RefCell};

#[derive(Debug,Clone,PartialEq)]
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

    pub fn clone_value(&self) -> ValTy {
        self.val.clone()
    }
}

#[derive(Debug,Clone, PartialEq)]
pub enum ValTy {
    op(Rc<RefCell<Op>>),
    con(i32),
    var(Rc<RefCell<UniqueVariable>>),
    adr(UniqueAddress),
    arr(UniqueArray),
    ret(RetRegister),
}

impl ValTy {
    pub fn to_string(&self) -> String {
        match &self {
            ValTy::op(op) => op.borrow().get_return_value(),
            ValTy::con(con) => {
                String::from("#") + &con.to_string()
            },
            ValTy::var(var) => {
                // Temporarily, I want it to output var name
                var.borrow().value_to_string()
                //var.borrow().get_ident()
            },
            ValTy::adr(adr) => adr.to_string(),
            ValTy::arr(arr) => arr.to_string(),
            ValTy::ret(ret) => ret.to_string(),
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

    // Useful for debugging or printing
    p_command: String,
}

impl Op {
    pub fn new(x_val: Option<Value>,
               y_val: Option<Value>,
               special_val: Option<String>,
               inst_number: usize,
               block_number: usize,
               inst_type: InstTy) -> Self
    {
        let mut p_command = String::new();

        Op { x_val , y_val , special_val, inst_number, block_number, inst_type, p_command }
    }

    pub fn build_op(x_val: Option<Value>,
                    y_val: Option<Value>,
                    special_val: Option<String>,
                    block_number: usize,
                    inst_number: usize,
                    inst_type: InstTy,
                    var_manager: &mut VariableManager) -> Op {
        // TODO : now I can add uses to variables as the operation is being built. should make the numbers far more accurate.
        match &x_val {
            Some(val) => {
                match val.get_value() {
                    ValTy::var(var) => {
                        var.borrow_mut().add_use(block_number, inst_number);
                    },
                    _ => {},
                }
            },
            None => {},
        }

        match &y_val {
            Some(val) => {
                match val.get_value() {
                    ValTy::var(var) => {
                        var.borrow_mut().add_use(block_number, inst_number);
                    }
                    _ => {},
                }
            },
            None => {},
        }

        Op::new(x_val,
                y_val,
                special_val,
                inst_number,
                block_number,
                inst_type)
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
                p_command = inst_type.to_string() + " " + &self.x_val.clone().unwrap().get_value().to_string();
            }
            // Op x y //
            InstTy::add | InstTy::sub | InstTy::mul |
            InstTy::div | InstTy::cmp | InstTy::adda |
            InstTy::bne | InstTy::beq | InstTy::ble |
            InstTy::blt | InstTy::bge | InstTy::bgt |
            InstTy::phi => {
                p_command = inst_type.to_string() + " " + &self.x_val.clone().unwrap().get_value().to_string()
                    + " " + &self.y_val.clone().unwrap().get_value().to_string();
            }
            // Op y //
            InstTy::load | InstTy::bra => {
                p_command = inst_type.to_string() + " " + &self.y_val.clone().unwrap().get_value().to_string();
            }
            // Op y x //
            InstTy::store | InstTy::mov => {
                p_command = inst_type.to_string() + " " + &self.y_val.clone().unwrap().get_value().to_string() +
                    " " + &self.x_val.clone().unwrap().get_value().to_string();
            }
            // Op [x] //
            InstTy::call => {
                p_command = String::from("call ");
                match &self.special_val {
                    Some(val_str) => {
                        p_command += &val_str;
                    },
                    None => {
                        panic!("Should probably always have a string value.");
                    },
                }
            }

            _ => { panic!("Error in Op construction, unexpected inst_type found."); }
        }

        p_command
    }

    pub fn get_return_value(&self) -> String {
        let string = String::from("(") + &self.inst_number.to_string() + ")";
        string
    }

    pub fn get_inst_block(&self) -> usize {
        self.block_number.clone()
    }

    pub fn get_inst_num(&self) -> usize { self.inst_number.clone() }

    pub fn inst_type(&self) -> &InstTy {
        &self.inst_type
    }

    pub fn var_cleanup(&mut self, var_to_clean: Value, replacement_var: Value) {
        //println!("x_val: {:?}, to_clean_val: {:?}", self.x_val.clone(), var_to_clean.clone());
        match self.x_val.clone() {
            Some(val) => {
                if val == var_to_clean {
                    println!("Clean cycle reaches x_val replacement for {}.", var_to_clean.get_value().to_string());
                    self.x_val = Some(replacement_var.clone());
                }
            },
            None => {
                // There is no variable to clean, pass through.
            }
        }

        //println!("y_val: {:?}, to_clean_val: {:?}", self.y_val.clone(), var_to_clean.clone());
        match self.y_val.clone() {
            Some(val) => {
                if val == var_to_clean {
                    println!("Clean cycle reaches y_val replacement for {}.", var_to_clean.get_value().to_string());
                    self.y_val = Some(replacement_var.clone());
                }
            },
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

impl PartialEq for Op {
    fn eq(&self, other: &Op) -> bool {
        if self.inst_type == other.inst_type {
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

#[derive(Clone, PartialEq, Eq, Hash)]
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
            InstTy::read => { String::from("read") },
            InstTy::end => { String::from("end") },
            InstTy::writeNL => { String::from("writeNL") },

            /// Op x ///
            InstTy::neg => { String::from("neg") },
            InstTy::write => { String::from("write") },
            InstTy::ret => { String::from("ret") },

            /// Op x y ///
            InstTy::add => { String::from("add") },
            InstTy::sub => { String::from("sub") },
            InstTy::mul => { String::from("mul") },
            InstTy::div => { String::from("div") },
            InstTy::cmp => { String::from("cmp") },
            InstTy::adda => { String::from("adda") },

            InstTy::bne => { String::from("bne") },
            InstTy::beq => { String::from("beq") },
            InstTy::ble => { String::from("ble") },
            InstTy::blt => { String::from("blt") },
            InstTy::bge => { String::from("bge") },
            InstTy::bgt => { String::from("bgt") },

            InstTy::phi => { String::from("phi") },

            /// Op y ///
            InstTy::load => { String::from("load") },
            InstTy::bra => { String::from("bra") },

            /// Op y x ///
            InstTy::store => { String::from("store") },
            InstTy::mov => { String::from("move") },

            /// Op [x] ///
            InstTy::call => { String::from("call") },

            _ => { panic!("Error occurred, was not a default type."); }
        }
    }
}