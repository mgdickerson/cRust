#[derive(Debug,Clone)]
pub struct Value {
    val: ValTy,
}

impl Value {
    pub fn new(val_ty: ValTy) -> Self {
        Value { val: val_ty }
    }

    // TODO : Clean up errors.
    pub fn get_value(&self) -> i32 {
        match &self.val {
            ValTy::inst(inst) => { -1 },    // Currently just an error
            ValTy::con(con) => { *con },
            ValTy::var(var) => { -1 },      // Currently just an error
            ValTy::reg(reg) => *reg,
        }
    }
}

#[derive(Debug,Clone)]
pub enum ValTy {
    inst(Inst),
    con(i32),
    var(String),
    reg(i32),
}

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

    fn debugPrint(&self) {
        println!("{}", self.p_command());
    }
}

/// neg ///

#[derive(Debug,Clone)]
pub struct Neg {
    x_val: Box<Value>,
    p_command: String,
}

impl Neg {
    pub fn new(x_val: Value) -> Self {
       Neg { x_val: Box::new(x_val.clone()), p_command: String::from("neg ") + &x_val.get_value().to_string() }
    }
}

impl Instruction for Neg {
    fn p_command(&self) -> &str {
        self.p_command.as_str()
    }
}

/// add ///

#[derive(Debug,Clone)]
pub struct Add {
    x_val: Box<Value>,
    y_val: Box<Value>,
    p_command: String,
}

impl Add {
    pub fn new(x_val: Value, y_val: Value) -> Self {
        let string = String::from("add ") + &x_val.get_value().to_string() + " " + &y_val.get_value().to_string();
        Add { x_val: Box::new(x_val), y_val: Box::new(y_val), p_command: string }
    }
}

impl Instruction for Add {
    fn p_command(&self) -> &str {
        self.p_command.as_str()
    }
}

/// sub ///

#[derive(Debug,Clone)]
pub struct Sub {
    x_val: Box<Value>,
    y_val: Box<Value>,
    p_command: String,
}

impl Sub {
    pub fn new(x_val: Value, y_val: Value) -> Self {
        let string = String::from("sub ") + &x_val.get_value().to_string() + " " + &y_val.get_value().to_string();
        Sub { x_val: Box::new(x_val), y_val: Box::new(y_val), p_command: string }
    }
}

impl Instruction for Sub {
    fn p_command(&self) -> &str {
        self.p_command.as_str()
    }
}

/// mul ///

#[derive(Debug,Clone)]
pub struct Mul {
    x_val: Box<Value>,
    y_val: Box<Value>,
    p_command: String,
}

impl Mul {
    pub fn new(x_val: Value, y_val: Value) -> Self {
        let string = String::from("mul ") + &x_val.get_value().to_string() + " " + &y_val.get_value().to_string();
        Mul { x_val: Box::new(x_val), y_val: Box::new(y_val), p_command: string }
    }
}

impl Instruction for Mul {
    fn p_command(&self) -> &str {
        self.p_command.as_str()
    }
}

/// div ///

#[derive(Debug,Clone)]
pub struct Div {
    x_val: Box<Value>,
    y_val: Box<Value>,
    p_command: String,
}

impl Div {
    pub fn new(x_val: Value, y_val: Value) -> Self {
        let string = String::from("div ") + &x_val.get_value().to_string() + " " + &y_val.get_value().to_string();
        Div { x_val: Box::new(x_val), y_val: Box::new(y_val), p_command: string }
    }
}

impl Instruction for Div {
    fn p_command(&self) -> &str {
        self.p_command.as_str()
    }
}

/// cmp ///

#[derive(Debug,Clone)]
pub struct Cmp {
    x_val: Box<Value>,
    y_val: Box<Value>,
    p_command: String,
}

impl Cmp {
    pub fn new(x_val: Value, y_val: Value) -> Self {
        let string = String::from("cmp ") + &x_val.get_value().to_string() + " " + &y_val.get_value().to_string();
        Cmp { x_val: Box::new(x_val), y_val: Box::new(y_val), p_command: string }
    }
}

impl Instruction for Cmp {
    fn p_command(&self) -> &str {
        self.p_command.as_str()
    }
}

/// adda ///

#[derive(Debug,Clone)]
pub struct Adda {
    x_val: Box<Value>,
    y_val: Box<Value>,
    p_command: String,
}

impl Adda {
    pub fn new(x_val: Value, y_val: Value) -> Self {
        let string = String::from("adda ") + &x_val.get_value().to_string() + " " + &y_val.get_value().to_string();
        Adda { x_val: Box::new(x_val), y_val: Box::new(y_val), p_command: string }
    }
}

impl Instruction for Adda {
    fn p_command(&self) -> &str {
        self.p_command.as_str()
    }
}

/// load ///

#[derive(Debug,Clone)]
pub struct Load {
    y_val: Box<Value>,
    p_command: String,
}

impl Load {
    pub fn new(x_val: Value, y_val: Value) -> Self {
        let string = String::from("load ") + &y_val.get_value().to_string();
        Load { y_val: Box::new(y_val), p_command: string }
    }
}

impl Instruction for Load {
    fn p_command(&self) -> &str {
        self.p_command.as_str()
    }
}

/// store ///

#[derive(Debug,Clone)]
pub struct Store {
    y_val: Box<Value>,
    x_val: Box<Value>,
    p_command: String,
}

impl Store {
    pub fn new(y_val: Value, x_val: Value) -> Self {
        let string = String::from("store ") + &y_val.get_value().to_string() + " " + &x_val.get_value().to_string();
        Store { y_val: Box::new(y_val), x_val: Box::new(x_val), p_command: string }
    }
}

impl Instruction for Store {
    fn p_command(&self) -> &str {
        self.p_command.as_str()
    }
}

/// move ///

#[derive(Debug,Clone)]
pub struct Move {
    y_val: Box<Value>,
    x_val: Box<Value>,
    p_command: String,
}

impl Move {
    pub fn new(y_val: Value, x_val: Value) -> Self {
        let string = String::from("move ") + &y_val.get_value().to_string() + " " + &x_val.get_value().to_string();
        Move { y_val: Box::new(y_val), x_val: Box::new(x_val), p_command: string }
    }
}

impl Instruction for Move {
    fn p_command(&self) -> &str {
        self.p_command.as_str()
    }
}

/// phi ///

#[derive(Debug,Clone)]
pub struct Phi {
    x_val: Box<Vec<Value>>,
    p_command: String,
}

impl Phi {
    pub fn new(x_val: Vec<Value>) -> Self {
        let mut string = String::from("phi := (");
        let mut first = true;
        for val in x_val.clone() {
            if !first { string += ", "; first = false; }
            string += &String::from(val.get_value().to_string());
        }
        string += ")";
        Phi { x_val: Box::new(x_val), p_command: string }
    }
}

impl Instruction for Phi {
    fn p_command(&self) -> &str {
        self.p_command.as_str()
    }
}

/// end ///

#[derive(Debug,Clone)]
pub struct End {
    p_command: String,
}

impl End {
    pub fn new() -> Self {
        let string = String::from("end ");
        End { p_command: string }
    }
}

impl Instruction for End {
    fn p_command(&self) -> &str {
        self.p_command.as_str()
    }
}

/// bra ///

#[derive(Debug,Clone)]
pub struct Bra {
    y_val: Box<Value>,
    p_command: String,
}

impl Bra {
    pub fn new(y_val: Value) -> Self {
        let string = String::from("bra ") + &y_val.get_value().to_string();
        Bra { y_val: Box::new(y_val), p_command: string }
    }
}

impl Instruction for Bra {
    fn p_command(&self) -> &str {
        self.p_command.as_str()
    }
}

/// bne ///

#[derive(Debug,Clone)]
pub struct BNE {
    x_val: Box<Value>,
    y_val: Box<Value>,
    p_command: String,
}

impl BNE {
    pub fn new(x_val: Value, y_val: Value) -> Self {
        let string = String::from("bne ") + &x_val.get_value().to_string() + " " + &y_val.get_value().to_string();
        BNE { x_val: Box::new(x_val), y_val: Box::new(y_val), p_command: string }
    }
}

impl Instruction for BNE {
    fn p_command(&self) -> &str {
        self.p_command.as_str()
    }
}

/// beq ///

#[derive(Debug,Clone)]
pub struct BEQ {
    x_val: Box<Value>,
    y_val: Box<Value>,
    p_command: String,
}

impl BEQ {
    pub fn new(x_val: Value, y_val: Value) -> Self {
        let string = String::from("beq ") + &x_val.get_value().to_string() + " " + &y_val.get_value().to_string();
        BEQ { x_val: Box::new(x_val), y_val: Box::new(y_val), p_command: string }
    }
}

impl Instruction for BEQ {
    fn p_command(&self) -> &str {
        self.p_command.as_str()
    }
}

/// ble ///

#[derive(Debug,Clone)]
pub struct BLE {
    x_val: Box<Value>,
    y_val: Box<Value>,
    p_command: String,
}

impl BLE {
    pub fn new(x_val: Value, y_val: Value) -> Self {
        let string = String::from("ble ") + &x_val.get_value().to_string() + " " + &y_val.get_value().to_string();
        BLE { x_val: Box::new(x_val), y_val: Box::new(y_val), p_command: string }
    }
}

impl Instruction for BLE {
    fn p_command(&self) -> &str {
        self.p_command.as_str()
    }
}

/// blt ///

#[derive(Debug,Clone)]
pub struct BLT {
    x_val: Box<Value>,
    y_val: Box<Value>,
    p_command: String,
}

impl BLT {
    pub fn new(x_val: Value, y_val: Value) -> Self {
        let string = String::from("blt ") + &x_val.get_value().to_string() + " " + &y_val.get_value().to_string();
        BLT { x_val: Box::new(x_val), y_val: Box::new(y_val), p_command: string }
    }
}

impl Instruction for BLT {
    fn p_command(&self) -> &str {
        self.p_command.as_str()
    }
}

/// bge ///

#[derive(Debug,Clone)]
pub struct BGE {
    x_val: Box<Value>,
    y_val: Box<Value>,
    p_command: String,
}

impl BGE {
    pub fn new(x_val: Value, y_val: Value) -> Self {
        let string = String::from("bge ") + &x_val.get_value().to_string() + " " + &y_val.get_value().to_string();
        BGE { x_val: Box::new(x_val), y_val: Box::new(y_val), p_command: string }
    }
}

impl Instruction for BGE {
    fn p_command(&self) -> &str {
        self.p_command.as_str()
    }
}

/// bgt ///

#[derive(Debug,Clone)]
pub struct BGT {
    x_val: Box<Value>,
    y_val: Box<Value>,
    p_command: String,
}

impl BGT {
    pub fn new(x_val: Value, y_val: Value) -> Self {
        let string = String::from("bgt ") + &x_val.get_value().to_string() + " " + &y_val.get_value().to_string();
        BGT { x_val: Box::new(x_val), y_val: Box::new(y_val), p_command: string }
    }
}

impl Instruction for BGT {
    fn p_command(&self) -> &str {
        self.p_command.as_str()
    }
}

/// read ///

#[derive(Debug,Clone)]
pub struct Read {
    p_command: String,
}

impl Read {
    pub fn new() -> Self {
        let string = String::from("read ");
        Read { p_command: string }
    }
}

impl Instruction for Read {
    fn p_command(&self) -> &str {
        self.p_command.as_str()
    }
}

/// write ///

#[derive(Debug,Clone)]
pub struct Write {
    x_val: Box<Value>,
    p_command: String,
}

impl Write {
    pub fn new(x_val: Value) -> Self {
        let string = String::from("write ") + &x_val.get_value().to_string();
        Write { x_val: Box::new(x_val), p_command: string }
    }
}

impl Instruction for Write {
    fn p_command(&self) -> &str {
        self.p_command.as_str()
    }
}

/// writenl ///

#[derive(Debug,Clone)]
pub struct WriteNL {
    p_command: String,
}

impl WriteNL {
    pub fn new() -> Self {
        let string = String::from("writenl ");
        WriteNL { p_command: string }
    }
}

impl Instruction for WriteNL {
    fn p_command(&self) -> &str {
        self.p_command.as_str()
    }
}

/// call ///

#[derive(Debug,Clone)]
pub struct Call {
    p_command: String,
}

impl Call {
    pub fn new() -> Self {
        let string = String::from("call ");
        Call { p_command: string }
    }
}

impl Instruction for Call {
    fn p_command(&self) -> &str {
        self.p_command.as_str()
    }
}

/// return ///

#[derive(Debug,Clone)]
pub struct Ret {
    x_val: Box<Value>,
    p_command: String,
}

impl Ret {
    pub fn new(x_val: Value) -> Self {
        let string = String::from("return ") + &x_val.get_value().to_string();
        Ret { x_val: Box::new(x_val), p_command: string }
    }
}

impl Instruction for Ret {
    fn p_command(&self) -> &str {
        self.p_command.as_str()
    }
}