use lib::IR::address_manager::{UniqueAddress, AddressType, AddressManager};
use lib::IR::array_manager::UniqueArray;
use lib::IR::ret_register::RetRegister;
use lib::IR::variable_manager::{UniqueVariable, VariableManager};

use super::{Rc, RefCell};
use lib::RegisterAllocator::RegisterAllocation;
use petgraph::graph::NodeIndex;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::collections::HashMap;

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
            }
            ValTy::con(con) => {
                con.clone().hash(state);
            }
            ValTy::var(var) => {
                0.hash(state);
            }
            ValTy::adr(adr) => {
                adr.to_string().hash(state);
            }
            ValTy::arr(arr) => {
                arr.to_string().hash(state);
            }
            ValTy::ret(ret) => {
                ret.to_string().hash(state);
            }
            ValTy::reg(reg) => {
                reg.to_string().hash(state);
            }
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
    register: Option<RegisterAllocation>,

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
        let p_command = String::new();

        Op {
            x_val,
            y_val,
            special_val,
            register: None,
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

    pub fn to_string(&self) -> String {
        let mut p_command = String::new();
        let inst_type = self.inst_type.clone();

        match &inst_type.clone() {
            // Op //
            InstTy::read | InstTy::end | InstTy::writeNL | InstTy::kill => {
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
            InstTy::load | InstTy::loadsp | InstTy::pload |
            InstTy::gload | InstTy::bra => {
                p_command = inst_type.to_string()
                    + " "
                    + &self.y_val.clone().unwrap().get_value().to_string();
            }
            // Op y x //
            InstTy::store | InstTy::mov |
            InstTy::pstore | InstTy::gstore | InstTy::spill => {
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

    pub fn op_to_register(&mut self, reg_map: & HashMap<usize, usize>) {
        if let Some(register) = reg_map.get(&self.inst_number) {
            self.register = Some(RegisterAllocation::allocate_register(register.clone()));
        }

        if let Some(x_val) = self.clone_x_val() {
            if let ValTy::op(x_op) = x_val.get_value().clone() {
                let register_num = reg_map.get(&x_op.borrow().get_inst_num()).unwrap().clone();
                self.x_val = Some(Value::new(ValTy::reg(RegisterAllocation::allocate_register(register_num))));
            }
        }

        if let Some(y_val) = self.clone_y_val() {
            if let ValTy::op(y_op) = y_val.get_value().clone() {
                let register_num = reg_map.get(&y_op.borrow().get_inst_num()).unwrap().clone();
                self.y_val = Some(Value::new(ValTy::reg(RegisterAllocation::allocate_register(register_num))));
            }
        }
    }

    pub fn address_to_const(&mut self, addr_manager: & AddressManager) {
        if let Some(x_val) = self.clone_x_val() {
            if let ValTy::adr(x_addr) = x_val.get_value().clone() {
                let addr_type = x_addr.get_type();
                match addr_type {
                    AddressType::g_reg => {
                        // Replace with the global register R30
                        self.x_val = Some(Value::new(ValTy::reg(RegisterAllocation::allocate_R30())));
                    },
                    AddressType::sp => {
                        self.x_val = Some(Value::new(ValTy::reg(RegisterAllocation::allocate_R29())));
                    },
                    _ => {
                        self.x_val = Some(Value::new(ValTy::con(addr_manager.get_assignment(&x_addr))));
                    },
                }
            }
        }

        if let Some(y_val) = self.clone_y_val() {
            if let ValTy::adr(y_addr) = y_val.get_value().clone() {
                let addr_type = y_addr.get_type();
                match addr_type {
                    AddressType::g_reg => {
                        self.y_val = Some(Value::new(ValTy::reg(RegisterAllocation::allocate_R30())));
                    },
                    AddressType::sp => {
                        self.y_val = Some(Value::new(ValTy::reg(RegisterAllocation::allocate_R29())));
                    },
                    _ => {
                        self.y_val = Some(Value::new(ValTy::con(addr_manager.get_assignment(&y_addr))));
                    },
                }
            }
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
        match self.register.clone() {
            Some(register) => {
                write!(f, "[{}] {} : {}; \\l ", self.inst_number, register.to_string(), self.to_string())
            },
            None => {
                write!(f, "[{}] {}; \\l ", self.inst_number, self.to_string())
            },
        }
    }
}

impl Hash for Op {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.inst_type.hash(state);
        match &self.x_val {
            Some(x_val) => {
                x_val.clone().hash(state);
            }
            None => {
                0.hash(state);
            }
        }
        match &self.y_val {
            Some(y_val) => {
                y_val.clone().hash(state);
            }
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
            let mut x_match_checked = false;
            let mut y_match_checked = false;

            let x_values = (self.clone_x_val(), other.clone_x_val());

            match x_values {
                (Some(self_val), Some(other_val)) => {
                    if let ValTy::op(self_op) = self_val.get_value() {
                        if let ValTy::op(other_op) = other_val.get_value() {
                            if self_op.borrow().get_inst_num() == other_op.borrow().get_inst_num() {
                                // both x ops match, check for y now.
                                x_match_checked = true;
                            } else {
                                return false;
                            }
                        } else {
                            return false;
                        }
                    }
                },
                (None, None) => {
                    x_match_checked = true;
                },
                _ => {
                    return false
                },
            }

            let y_values = (self.clone_y_val(), other.clone_y_val());

            match y_values {
                (Some(self_val), Some(other_val)) => {
                    if let ValTy::op(self_op) = self_val.get_value() {
                        if let ValTy::op(other_op) = other_val.get_value() {
                            if self_op.borrow().get_inst_num() == other_op.borrow().get_inst_num() {
                                // both x ops match, check for y now.
                                y_match_checked = true;
                            } else {
                                return false;
                            }
                        } else {
                            return false;
                        }
                    }
                },
                (None, None) => {
                    if x_match_checked == true {
                        return true
                    } else {
                        y_match_checked = true;
                    }
                },
                _ => {
                    return false
                },
            }

            // First check for the case where x is an op, but y is not.
            if x_match_checked {
                if self.y_val == other.y_val {
                    return true;
                }
            }

            // Check for case where y is an op, but x is not.
            if y_match_checked {
                if self.x_val == other.x_val {
                    return true;
                }
            }

            // Case where neither x or y is an op
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

// TODO : Need to add some instructions to make function calls easier
// Ex: - param (var)
//     - affect-global (var)
//     - All functions need a prologue and epilogue (This can be done in codegen)

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum InstTy {
    /// Op ///
    read,
    end,
    writeNL,
    kill,

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
    loadsp,
    pload,
    gload,
    bra,

    /// Op y x ///
    store,
    mov,

    spill,

    // Indicate that function store register
    // value for function parameter.
    // Same layout as store.
    // param (x) location (y) value
    pstore,

    // Indicate that function store register value
    // for a global affected within the function.
    // Same layout as store.
    // global (x) location (y) value
    gstore,

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
            InstTy::kill => String::from("kill"),

            /// Op x ///
            InstTy::neg => String::from("neg"),
            InstTy::write => String::from("write"),
            InstTy::ret => String::from("ret"),

            /// Op x y ///
            InstTy::add => String::from("add"),
            InstTy::spill => String::from("spill"),
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
            InstTy::loadsp => String::from("load_spill"),
            InstTy::bra => String::from("bra"),

            InstTy::gload => String::from("gload"),

            // This will only be used for the return result
            InstTy::pload => String::from("pload"),

            /// Op y x ///
            InstTy::store => String::from("store"),
            InstTy::mov => String::from("move"),

            InstTy::pstore => String::from("pstore"),
            InstTy::gstore => String::from("gstore"),

            /// Op [x] ///
            InstTy::call => String::from("call"),

            _ => {
                panic!("Error occurred, was not a default type.");
            }
        }
    }
}
