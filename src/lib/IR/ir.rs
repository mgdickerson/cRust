#[derive(Debug,Clone)]
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

#[derive(Debug,Clone)]
pub enum ValTy {
    op(Op),
    con(i32),
    var(String),
    reg(i32),
    arr(String),
}

impl ValTy {
    pub fn to_string(&self) -> String {
        match &self {
            ValTy::op(op) => op.get_return_value(),
            ValTy::con(con) => {
                String::from("#") + &con.to_string()
            },
            ValTy::var(var) => var.clone(),
            ValTy::reg(reg) => reg.to_string(),
            ValTy::arr(arr) => arr.clone(),
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
            InstTy::blt | InstTy::bge | InstTy::bgt => {
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
            // TODO : This will need to be changed, as Phi will likely not know its operands at start.
            InstTy::phi => {
                p_command = String::from("phi := (");
                let mut first = true;
                for val in special_val_string.unwrap().clone() {
                    if !first { p_command += ", "; }
                    p_command += &String::from(val.get_value().to_string());
                    first = false;
                }
                p_command += ")";
            }
            InstTy::call => {
                p_command = String::from("call ");
                let mut first = true;
                match special_val_string {
                    Some(val_vec) => {},
                    None => {},
                }
            }

            _ => { panic!("Error in Op construction, unexpected inst_type found."); }
        }

        Op { x_val, y_val, special_val, inst_number, block_number, inst_type, p_command }
    }

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
}

impl std::fmt::Debug for Op {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "({}): {}; \\l ", self.inst_number, self.p_command)
    }
}

#[derive(Clone)]
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

    /// Op y ///
    load,
    bra,

    /// Op y x ///
    store,
    mov,

    /// Op [x] ///
    phi,
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

            /// Op y ///
            InstTy::load => { String::from("load") },
            InstTy::bra => { String::from("bra") },

            /// Op y x ///
            InstTy::store => { String::from("store") },
            InstTy::mov => { String::from("move") },

            /// Op [x] ///
            InstTy::phi => { String::from("phi") },
            InstTy::call => { String::from("call") },

            _ => { panic!("Error occurred, was not a default type."); }
        }
    }
}

pub struct InstTracker {
    inst_number: usize,
}

impl InstTracker {
    pub fn new() -> InstTracker {
        InstTracker { inst_number: 0 }
    }

    pub fn increment(&mut self) {
        self.inst_number += 1;
    }

    pub fn get(&self) -> usize {
        self.inst_number.clone()
    }
}

/*
#[derive(Debug,Clone)]
pub enum Inst {
    neg(Neg),
    add(Add),
    adda(Adda),
    sub(Sub),
    mul(Mul),
    div(Div),
    cmp(Cmp),
    load(Load),
    store(Store),
    mov(Move),
    phi(Phi),
    end(End),
    bra(Bra),
    bne(BNE),
    beq(BEQ),
    ble(BLE),
    blt(BLT),
    bge(BGE),
    bgt(BGT),
    read(Read),
    write(Write),
    writeNL(WriteNL),
    call(Call),
    ret(Ret),
}

// TODO : works with self, instead of &mut self
impl Inst {
    pub fn get_inst(self) -> Option<Box<Instruction>> {
        match self {
            Inst::neg(val) => { Some(Box::new(val)) },
            Inst::add(val) => { Some(Box::new(val)) },
            Inst::adda(val) => { Some(Box::new(val)) },
            Inst::sub(val) => { Some(Box::new(val)) },
            Inst::mul(val) => { Some(Box::new(val)) },
            Inst::div(val) => { Some(Box::new(val)) },
            Inst::cmp(val) => { Some(Box::new(val)) },
            Inst::load(val) => { Some(Box::new(val)) },
            Inst::store(val) => { Some(Box::new(val)) },
            Inst::mov(val) => { Some(Box::new(val)) },
            Inst::phi(val) => { Some(Box::new(val)) },
            Inst::end(val) => { Some(Box::new(val)) },
            Inst::bra(val) => { Some(Box::new(val)) },
            Inst::bne(val) => { Some(Box::new(val)) },
            Inst::beq(val) => { Some(Box::new(val)) },
            Inst::ble(val) => { Some(Box::new(val)) },
            Inst::blt(val) => { Some(Box::new(val)) },
            Inst::bge(val) => { Some(Box::new(val)) },
            Inst::bgt(val) => { Some(Box::new(val)) },
            Inst::read(val) => { Some(Box::new(val)) },
            Inst::write(val) => { Some(Box::new(val)) },
            Inst::writeNL(val) => { Some(Box::new(val)) },
            Inst::call(val) => { Some(Box::new(val)) },
            Inst::ret(val) => { Some(Box::new(val)) },
            _ => None,
        }
    }
}


pub trait Instruction {
    fn p_command(&self) -> &str;

    fn inst_number(&self) -> &usize;

    fn print_command(&self) {
        println!("{}", self.p_command());
    }

    fn debug_print(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}; \\l", self.p_command())
    }
}

impl std::fmt::Debug for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "({}): {}; \\l ", self.inst_number(), self.p_command())
    }
}

/// neg ///

#[derive(Debug,Clone)]
pub struct Neg {
    x_val: Box<Value>,
    p_command: String,
    inst_number: usize,
}

impl Neg {
    pub fn new(x_val: Value, inst_number: usize) -> Self {
       Neg { x_val: Box::new(x_val.clone()), p_command: String::from("neg ") + &x_val.get_value().to_string(), inst_number }
    }
}

impl Instruction for Neg {
    fn p_command(&self) -> &str {
        self.p_command.as_str()
    }

    fn inst_number(&self) -> &usize {
        &self.inst_number
    }
}

/// add ///

#[derive(Debug,Clone)]
pub struct Add {
    x_val: Box<Value>,
    y_val: Box<Value>,
    p_command: String,
    inst_number: usize,
}

impl Add {
    pub fn new(x_val: Value, y_val: Value, inst_number: usize) -> Self {
        let string = String::from("add ") + &x_val.get_value().to_string() + " " + &y_val.get_value().to_string();
        Add { x_val: Box::new(x_val), y_val: Box::new(y_val), p_command: string, inst_number }
    }
}

impl Instruction for Add {
    fn p_command(&self) -> &str {
        self.p_command.as_str()
    }

    fn inst_number(&self) -> &usize {
        &self.inst_number
    }
}

/// sub ///

#[derive(Debug,Clone)]
pub struct Sub {
    x_val: Box<Value>,
    y_val: Box<Value>,
    p_command: String,
    inst_number: usize,
}

impl Sub {
    pub fn new(x_val: Value, y_val: Value, inst_number: usize) -> Self {
        let string = String::from("sub ") + &x_val.get_value().to_string() + " " + &y_val.get_value().to_string();
        Sub { x_val: Box::new(x_val), y_val: Box::new(y_val), p_command: string, inst_number }
    }
}

impl Instruction for Sub {
    fn p_command(&self) -> &str {
        self.p_command.as_str()
    }

    fn inst_number(&self) -> &usize {
        &self.inst_number
    }
}

/// mul ///

#[derive(Debug,Clone)]
pub struct Mul {
    x_val: Box<Value>,
    y_val: Box<Value>,
    p_command: String,
    inst_number: usize,
}

impl Mul {
    pub fn new(x_val: Value, y_val: Value, inst_number: usize) -> Self {
        let string = String::from("mul ") + &x_val.get_value().to_string() + " " + &y_val.get_value().to_string();
        Mul { x_val: Box::new(x_val), y_val: Box::new(y_val), p_command: string, inst_number }
    }
}

impl Instruction for Mul {
    fn p_command(&self) -> &str {
        self.p_command.as_str()
    }

    fn inst_number(&self) -> &usize {
        &self.inst_number
    }
}

/// div ///

#[derive(Debug,Clone)]
pub struct Div {
    x_val: Box<Value>,
    y_val: Box<Value>,
    p_command: String,
    inst_number: usize,
}

impl Div {
    pub fn new(x_val: Value, y_val: Value, inst_number: usize) -> Self {
        let string = String::from("div ") + &x_val.get_value().to_string() + " " + &y_val.get_value().to_string();
        Div { x_val: Box::new(x_val), y_val: Box::new(y_val), p_command: string, inst_number }
    }
}

impl Instruction for Div {
    fn p_command(&self) -> &str {
        self.p_command.as_str()
    }

    fn inst_number(&self) -> &usize {
        &self.inst_number
    }
}

/// cmp ///

#[derive(Debug,Clone)]
pub struct Cmp {
    x_val: Box<Value>,
    y_val: Box<Value>,
    p_command: String,
    inst_number: usize,
}

impl Cmp {
    pub fn new(x_val: Value, y_val: Value, inst_number: usize) -> Self {
        let string = String::from("cmp ") + &x_val.get_value().to_string() + " " + &y_val.get_value().to_string();
        Cmp { x_val: Box::new(x_val), y_val: Box::new(y_val), p_command: string, inst_number }
    }
}

impl Instruction for Cmp {
    fn p_command(&self) -> &str {
        self.p_command.as_str()
    }

    fn inst_number(&self) -> &usize {
        &self.inst_number
    }
}

/// adda ///

#[derive(Debug,Clone)]
pub struct Adda {
    x_val: Box<Value>,
    y_val: Box<Value>,
    p_command: String,
    inst_number: usize,
}

impl Adda {
    pub fn new(x_val: Value, y_val: Value, inst_number: usize) -> Self {
        let string = String::from("adda ") + &x_val.get_value().to_string() + " " + &y_val.get_value().to_string();
        Adda { x_val: Box::new(x_val), y_val: Box::new(y_val), p_command: string, inst_number }
    }
}

impl Instruction for Adda {
    fn p_command(&self) -> &str {
        self.p_command.as_str()
    }

    fn inst_number(&self) -> &usize {
        &self.inst_number
    }
}

/// load ///

#[derive(Debug,Clone)]
pub struct Load {
    y_val: Box<Value>,
    p_command: String,
    inst_number: usize,
}

impl Load {
    pub fn new(x_val: Value, y_val: Value, inst_number: usize) -> Self {
        let string = String::from("load ") + &y_val.get_value().to_string();
        Load { y_val: Box::new(y_val), p_command: string, inst_number }
    }
}

impl Instruction for Load {
    fn p_command(&self) -> &str {
        self.p_command.as_str()
    }

    fn inst_number(&self) -> &usize {
        &self.inst_number
    }
}

/// store ///

#[derive(Debug,Clone)]
pub struct Store {
    y_val: Box<Value>,
    x_val: Box<Value>,
    p_command: String,
    inst_number: usize,
}

impl Store {
    pub fn new(y_val: Value, x_val: Value, inst_number: usize) -> Self {
        let string = String::from("store ") + &y_val.get_value().to_string() + " " + &x_val.get_value().to_string();
        Store { y_val: Box::new(y_val), x_val: Box::new(x_val), p_command: string, inst_number }
    }
}

impl Instruction for Store {
    fn p_command(&self) -> &str {
        self.p_command.as_str()
    }

    fn inst_number(&self) -> &usize {
        &self.inst_number
    }
}

/// move ///

#[derive(Debug,Clone)]
pub struct Move {
    y_val: Box<Value>,
    x_val: Box<Value>,
    p_command: String,
    inst_number: usize,
}

impl Move {
    pub fn new(y_val: Value, x_val: Value, inst_number: usize) -> Self {
        let string = String::from("move ") + &y_val.get_value().to_string() + " " + &x_val.get_value().to_string();
        Move { y_val: Box::new(y_val), x_val: Box::new(x_val), p_command: string, inst_number }
    }
}

impl Instruction for Move {
    fn p_command(&self) -> &str {
        self.p_command.as_str()
    }

    fn inst_number(&self) -> &usize {
        &self.inst_number
    }
}

/// phi ///

#[derive(Debug,Clone)]
pub struct Phi {
    x_val: Box<Vec<Value>>,
    p_command: String,
    inst_number: usize,
}

impl Phi {
    pub fn new(x_val: Vec<Value>, inst_number: usize) -> Self {
        let mut string = String::from("phi := (");
        let mut first = true;
        for val in x_val.clone() {
            if !first { string += ", "; first = false; }
            string += &String::from(val.get_value().to_string());
        }
        string += ")";
        Phi { x_val: Box::new(x_val), p_command: string, inst_number }
    }
}

impl Instruction for Phi {
    fn p_command(&self) -> &str {
        self.p_command.as_str()
    }

    fn inst_number(&self) -> &usize {
        &self.inst_number
    }
}

/// end ///

#[derive(Debug,Clone)]
pub struct End {
    p_command: String,
    inst_number: usize,
}

impl End {
    pub fn new(inst_number: usize) -> Self {
        let string = String::from("end ");
        End { p_command: string, inst_number }
    }
}

impl Instruction for End {
    fn p_command(&self) -> &str {
        self.p_command.as_str()
    }

    fn inst_number(&self) -> &usize {
        &self.inst_number
    }
}

/// bra ///

#[derive(Debug,Clone)]
pub struct Bra {
    y_val: Box<Value>,
    p_command: String,
    inst_number: usize
}

impl Bra {
    pub fn new(y_val: Value, inst_number: usize) -> Self {
        let string = String::from("bra ") + &y_val.get_value().to_string();
        Bra { y_val: Box::new(y_val), p_command: string, inst_number }
    }
}

impl Instruction for Bra {
    fn p_command(&self) -> &str {
        self.p_command.as_str()
    }

    fn inst_number(&self) -> &usize {
        &self.inst_number
    }
}

/// bne ///

#[derive(Debug,Clone)]
pub struct BNE {
    x_val: Box<Value>,
    y_val: Box<Value>,
    p_command: String,
    inst_number: usize,
}

impl BNE {
    pub fn new(x_val: Value, y_val: Value, inst_number: usize) -> Self {
        let string = String::from("bne ") + &x_val.get_value().to_string() + " " + &y_val.get_value().to_string();
        BNE { x_val: Box::new(x_val), y_val: Box::new(y_val), p_command: string, inst_number }
    }
}

impl Instruction for BNE {
    fn p_command(&self) -> &str {
        self.p_command.as_str()
    }

    fn inst_number(&self) -> &usize {
        &self.inst_number
    }
}

/// beq ///

#[derive(Debug,Clone)]
pub struct BEQ {
    x_val: Box<Value>,
    y_val: Box<Value>,
    p_command: String,
    inst_number: usize,
}

impl BEQ {
    pub fn new(x_val: Value, y_val: Value, inst_number: usize) -> Self {
        let string = String::from("beq ") + &x_val.get_value().to_string() + " " + &y_val.get_value().to_string();
        BEQ { x_val: Box::new(x_val), y_val: Box::new(y_val), p_command: string, inst_number }
    }
}

impl Instruction for BEQ {
    fn p_command(&self) -> &str {
        self.p_command.as_str()
    }

    fn inst_number(&self) -> &usize {
        &self.inst_number
    }
}

/// ble ///

#[derive(Debug,Clone)]
pub struct BLE {
    x_val: Box<Value>,
    y_val: Box<Value>,
    p_command: String,
    inst_number: usize,
}

impl BLE {
    pub fn new(x_val: Value, y_val: Value, inst_number: usize) -> Self {
        let string = String::from("ble ") + &x_val.get_value().to_string() + " " + &y_val.get_value().to_string();
        BLE { x_val: Box::new(x_val), y_val: Box::new(y_val), p_command: string, inst_number }
    }
}

impl Instruction for BLE {
    fn p_command(&self) -> &str {
        self.p_command.as_str()
    }

    fn inst_number(&self) -> &usize {
        &self.inst_number
    }
}

/// blt ///

#[derive(Debug,Clone)]
pub struct BLT {
    x_val: Box<Value>,
    y_val: Box<Value>,
    p_command: String,
    inst_number: usize,
}

impl BLT {
    pub fn new(x_val: Value, y_val: Value, inst_number: usize) -> Self {
        let string = String::from("blt ") + &x_val.get_value().to_string() + " " + &y_val.get_value().to_string();
        BLT { x_val: Box::new(x_val), y_val: Box::new(y_val), p_command: string, inst_number }
    }
}

impl Instruction for BLT {
    fn p_command(&self) -> &str {
        self.p_command.as_str()
    }

    fn inst_number(&self) -> &usize {
        &self.inst_number
    }
}

/// bge ///

#[derive(Debug,Clone)]
pub struct BGE {
    x_val: Box<Value>,
    y_val: Box<Value>,
    p_command: String,
    inst_number: usize,
}

impl BGE {
    pub fn new(x_val: Value, y_val: Value, inst_number: usize) -> Self {
        let string = String::from("bge ") + &x_val.get_value().to_string() + " " + &y_val.get_value().to_string();
        BGE { x_val: Box::new(x_val), y_val: Box::new(y_val), p_command: string, inst_number }
    }
}

impl Instruction for BGE {
    fn p_command(&self) -> &str {
        self.p_command.as_str()
    }

    fn inst_number(&self) -> &usize {
        &self.inst_number
    }
}

/// bgt ///

#[derive(Debug,Clone)]
pub struct BGT {
    x_val: Box<Value>,
    y_val: Box<Value>,
    p_command: String,
    inst_number: usize,
}

impl BGT {
    pub fn new(x_val: Value, y_val: Value, inst_number: usize) -> Self {
        let string = String::from("bgt ") + &x_val.get_value().to_string() + " " + &y_val.get_value().to_string();
        BGT { x_val: Box::new(x_val), y_val: Box::new(y_val), p_command: string, inst_number }
    }
}

impl Instruction for BGT {
    fn p_command(&self) -> &str {
        self.p_command.as_str()
    }

    fn inst_number(&self) -> &usize {
        &self.inst_number
    }
}

/// read ///

#[derive(Debug,Clone)]
pub struct Read {
    p_command: String,
    inst_number: usize,
}

impl Read {
    pub fn new(inst_number: usize) -> Self {
        let string = String::from("read ");
        Read { p_command: string, inst_number }
    }
}

impl Instruction for Read {
    fn p_command(&self) -> &str {
        self.p_command.as_str()
    }

    fn inst_number(&self) -> &usize {
        &self.inst_number
    }
}

/// write ///

#[derive(Debug,Clone)]
pub struct Write {
    x_val: Box<Value>,
    p_command: String,
    inst_number: usize,
}

impl Write {
    pub fn new(x_val: Value, inst_number: usize) -> Self {
        let string = String::from("write ") + &x_val.get_value().to_string();
        Write { x_val: Box::new(x_val), p_command: string, inst_number }
    }
}

impl Instruction for Write {
    fn p_command(&self) -> &str {
        self.p_command.as_str()
    }

    fn inst_number(&self) -> &usize {
        &self.inst_number
    }
}

/// writenl ///

#[derive(Debug,Clone)]
pub struct WriteNL {
    p_command: String,
    inst_number: usize,
}

impl WriteNL {
    pub fn new(inst_number: usize) -> Self {
        let string = String::from("writenl ");
        WriteNL { p_command: string, inst_number }
    }
}

impl Instruction for WriteNL {
    fn p_command(&self) -> &str {
        self.p_command.as_str()
    }

    fn inst_number(&self) -> &usize {
        &self.inst_number
    }
}

/// call ///

#[derive(Debug,Clone)]
pub struct Call {
    p_command: String,
    inst_number: usize,
}

impl Call {
    pub fn new(inst_number: usize) -> Self {
        let string = String::from("call ");
        Call { p_command: string, inst_number }
    }
}

impl Instruction for Call {
    fn p_command(&self) -> &str {
        self.p_command.as_str()
    }

    fn inst_number(&self) -> &usize {
        &self.inst_number
    }
}

/// return ///

#[derive(Debug,Clone)]
pub struct Ret {
    x_val: Box<Value>,
    p_command: String,
    inst_number: usize,
}

impl Ret {
    pub fn new(x_val: Value, inst_number: usize) -> Self {
        let string = String::from("return ") + &x_val.get_value().to_string();
        Ret { x_val: Box::new(x_val), p_command: string, inst_number }
    }
}

impl Instruction for Ret {
    fn p_command(&self) -> &str {
        self.p_command.as_str()
    }

    fn inst_number(&self) -> &usize {
        &self.inst_number
    }
}
*/