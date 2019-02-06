use lib::IR::variable_manager::UniqueVariable;
use lib::IR::array_manager::UniqueArray;
use lib::IR::address_manager::UniqueAddress;

#[derive(Debug,Clone,PartialEq)]
pub struct Value {
    val: ValTy,
}

impl Value {
    pub fn new(val_ty: ValTy) -> Self {
        Value { val: val_ty }
    }

    // TODO : Clean up errors.
    pub fn get_value(&self) -> &ValTy {
        &self.val
    }
}

#[derive(Debug,Clone, PartialEq)]
pub enum ValTy {
    op(Op),
    con(i32),
    var(UniqueVariable),
    adr(UniqueAddress),
    arr(UniqueArray),
    // TODO : Add register type specifically for return
}

impl ValTy {
    pub fn to_string(&self) -> String {
        match &self {
            ValTy::op(op) => op.get_return_value(),
            ValTy::con(con) => {
                String::from("#") + &con.to_string()
            },
            ValTy::var(var) => {
                // Temporarily, I want it to output var name
                var.value_to_string()
                //var.get_ident()
            },
            ValTy::adr(adr) => adr.to_string(),
            ValTy::arr(arr) => arr.to_string(),
        }
    }
}

#[derive(Clone)]
pub struct Op {
    // Value Operands
    x_val: Option<Box<Value>>,
    y_val: Option<Box<Value>>,
    special_val: Option<Vec<Box<Value>>>,

    // General Information about self
    inst_number: usize,
    block_number: usize,
    inst_type: InstTy,

    // Useful for debugging or printing
    p_command: String,
}

impl Op {
    pub fn new(x_val: Option<Box<Value>>,
               y_val: Option<Box<Value>>,
               special_val: Option<Vec<Box<Value>>>,
               inst_number: usize,
               block_number: usize,
               inst_type: InstTy) -> Self
    {
        let mut p_command = String::new();
        let x_val_string = x_val.clone();
        let y_val_string = y_val.clone();
        let special_val_string = special_val.clone();

        match inst_type.clone() {
            // Op //
            InstTy::read | InstTy::end | InstTy::writeNL => {
                p_command = inst_type.to_string();
            }
            // Op x //
            InstTy::neg | InstTy::write | InstTy::ret => {
                p_command = inst_type.to_string() + " " + &x_val_string.unwrap().get_value().to_string();
            }
            // Op x y //
            InstTy::add | InstTy::sub | InstTy::mul |
            InstTy::div | InstTy::cmp | InstTy::adda |
            InstTy::bne | InstTy::beq | InstTy::ble |
            InstTy::blt | InstTy::bge | InstTy::bgt |
            InstTy::phi => {
                p_command = inst_type.to_string() + " " + &x_val_string.unwrap().get_value().to_string()
                    + " " + &y_val_string.unwrap().get_value().to_string();
            }
            // Op y //
            InstTy::load | InstTy::bra => {
                p_command = inst_type.to_string() + " " + &y_val_string.unwrap().get_value().to_string();
            }
            // Op y x //
            InstTy::store | InstTy::mov => {
                p_command = inst_type.to_string() + " " + &y_val_string.unwrap().get_value().to_string() +
                    " " + &x_val_string.unwrap().get_value().to_string();
            }
            // Op [x] //
            InstTy::call => {
                p_command = String::from("call ");
                match special_val_string {
                    // TODO : Still need to fix this.
                    Some(val_vec) => {},
                    None => {},
                }
            }

            _ => { panic!("Error in Op construction, unexpected inst_type found."); }
        }

        Op { x_val, y_val, special_val, inst_number, block_number, inst_type, p_command }
    }

    // TODO : Need to make another generalization here. Instead of adding use when variable
    // TODO : is called, add use when variable is used in one of these build ops.
    pub fn build_op(inst_number: usize, block_number: usize, inst_type: InstTy) -> Op {
        Op::new(None,None,None,inst_number,block_number,inst_type)
    }

    pub fn build_op_x(x_val: Value, inst_number: usize, block_number: usize, inst_type: InstTy) -> Op {
        Op::new(Some(Box::new(x_val)),None,None,inst_number,block_number,inst_type)
    }

    pub fn build_op_x_y(x_val: Value, y_val: Value, inst_number: usize, block_number: usize, inst_type: InstTy) -> Op {
        Op::new(Some(Box::new(x_val)),
                Some(Box::new(y_val)),
                None,
                inst_number,
                block_number,
                inst_type)
    }

    pub fn build_op_y(y_val: Value, inst_number: usize, block_number: usize, inst_type: InstTy) -> Op {
        Op::new(None, Some(Box::new(y_val)), None, inst_number, block_number, inst_type)
    }

    pub fn build_spec_op(special_val: Vec<Box<Value>>, inst_number: usize, block_number: usize, inst_type: InstTy) -> Op {
        Op::new(None,None,Some(special_val),inst_number,block_number,inst_type)
    }

    pub fn to_string(&self) -> String {
        self.p_command.clone()
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
        let mut x_val_string = None;
        let mut y_val_string = None;

        match self.x_val.clone() {
            Some(val) => {
                if *val == var_to_clean {
                    self.x_val = Some(Box::new(replacement_var.clone()));
                }
                x_val_string = Some(self.x_val.clone().unwrap());
            },
            None => {
                // There is no variable to clean, pass through.
            }
        }

        match self.y_val.clone() {
            Some(val) => {
                if *val == var_to_clean {
                    self.y_val = Some(Box::new(replacement_var.clone()));
                }
                y_val_string = Some(self.y_val.clone().unwrap());
            },
            None => {
                // There is no variable to clean, pass through.
            }
        }

        // Update p_command for printing purposes.
        match self.inst_type.clone() {
            // Op //
            InstTy::read | InstTy::end | InstTy::writeNL => {
                self.p_command = self.inst_type.to_string();
            }
            // Op x //
            InstTy::neg | InstTy::write | InstTy::ret => {
                self.p_command = self.inst_type.to_string() + " " + &x_val_string.unwrap().get_value().to_string();
            }
            // Op x y //
            InstTy::add | InstTy::sub | InstTy::mul |
            InstTy::div | InstTy::cmp | InstTy::adda |
            InstTy::bne | InstTy::beq | InstTy::ble |
            InstTy::blt | InstTy::bge | InstTy::bgt |
            InstTy::phi => {
                self.p_command = self.inst_type.to_string() + " " + &x_val_string.unwrap().get_value().to_string()
                    + " " + &y_val_string.unwrap().get_value().to_string();
            }
            // Op y //
            InstTy::load | InstTy::bra => {
                self.p_command = self.inst_type.to_string() + " " + &y_val_string.unwrap().get_value().to_string();
            }
            // Op y x //
            InstTy::store | InstTy::mov => {
                self.p_command = self.inst_type.to_string() + " " + &y_val_string.unwrap().get_value().to_string() +
                    " " + &x_val_string.unwrap().get_value().to_string();
            }
            // Op [x] //
            InstTy::call => {
                // TODO : Handle call
            }
        }
    }
}

impl std::fmt::Debug for Op {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "({}): {}; \\l ", self.inst_number, self.p_command)
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

    /// Op [x] ///
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