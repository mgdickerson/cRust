#[derive(Debug,Clone)]
pub struct Operand {
    value: String,
    val_type: OpType,
}

impl Operand {
    pub fn new(value: String, val_type: OpType) -> Self {
        Operand { value, val_type }
    }

    pub fn get_value(&self) -> String {
        self.value.clone()
    }

    pub fn get_type(&self) -> OpType {
        self.val_type.clone()
    }
}

#[derive(Debug,Clone)]
pub enum OpType {
    constant,
    variable,
    destination,
}

pub trait Inst {
    fn p_command(&self) -> &str;

    fn debugPrint(&self) {
        println!("{}", self.p_command());
    }
}

/// neg ///

pub struct Neg {
    x_val: Operand,
    p_command: String,
}

impl Neg {
    pub fn new(x_val: Operand) -> Self {
       Neg { x_val: x_val.clone(), p_command: String::from("neg ") + &x_val.get_value() }
    }
}

impl Inst for Neg {
    fn p_command(&self) -> &str {
        self.p_command.as_str()
    }
}

/// add ///

pub struct Add {
    x_val: Operand,
    y_val: Operand,
    p_command: String,
}

impl Add {
    pub fn new(x_val: Operand, y_val: Operand) -> Self {
        let string = String::from("add ") + &x_val.get_value() + " " + &y_val.get_value();
        Add { x_val, y_val, p_command: string }
    }
}

impl Inst for Add {
    fn p_command(&self) -> &str {
        self.p_command.as_str()
    }
}

/// sub ///

pub struct Sub {
    x_val: Operand,
    y_val: Operand,
    p_command: String,
}

impl Sub {
    pub fn new(x_val: Operand, y_val: Operand) -> Self {
        let string = String::from("sub ") + &x_val.get_value() + " " + &y_val.get_value();
        Sub { x_val, y_val, p_command: string }
    }
}

impl Inst for Sub {
    fn p_command(&self) -> &str {
        self.p_command.as_str()
    }
}

/// mul ///

pub struct Mul {
    x_val: Operand,
    y_val: Operand,
    p_command: String,
}

impl Mul {
    pub fn new(x_val: Operand, y_val: Operand) -> Self {
        let string = String::from("mul ") + &x_val.get_value() + " " + &y_val.get_value();
        Mul { x_val, y_val, p_command: string }
    }
}

impl Inst for Mul {
    fn p_command(&self) -> &str {
        self.p_command.as_str()
    }
}

/// div ///

pub struct Div {
    x_val: Operand,
    y_val: Operand,
    p_command: String,
}

impl Div {
    pub fn new(x_val: Operand, y_val: Operand) -> Self {
        let string = String::from("div ") + &x_val.get_value() + " " + &y_val.get_value();
        Div { x_val, y_val, p_command: string }
    }
}

impl Inst for Div {
    fn p_command(&self) -> &str {
        self.p_command.as_str()
    }
}

/// cmp ///

pub struct Cmp {
    x_val: Operand,
    y_val: Operand,
    p_command: String,
}

impl Cmp {
    pub fn new(x_val: Operand, y_val: Operand) -> Self {
        let string = String::from("cmp ") + &x_val.get_value() + " " + &y_val.get_value();
        Cmp { x_val, y_val, p_command: string }
    }
}

impl Inst for Cmp {
    fn p_command(&self) -> &str {
        self.p_command.as_str()
    }
}

/// adda ///

pub struct Adda {
    x_val: Operand,
    y_val: Operand,
    p_command: String,
}

impl Adda {
    pub fn new(x_val: Operand, y_val: Operand) -> Self {
        let string = String::from("adda ") + &x_val.get_value() + " " + &y_val.get_value();
        Adda { x_val, y_val, p_command: string }
    }
}

impl Inst for Adda {
    fn p_command(&self) -> &str {
        self.p_command.as_str()
    }
}

/// load ///

pub struct Load {
    y_val: Operand,
    p_command: String,
}

impl Load {
    pub fn new(x_val: Operand, y_val: Operand) -> Self {
        let string = String::from("load ") + &y_val.get_value();
        Load { y_val, p_command: string }
    }
}

impl Inst for Load {
    fn p_command(&self) -> &str {
        self.p_command.as_str()
    }
}

/// store ///

pub struct Store {
    y_val: Operand,
    x_val: Operand,
    p_command: String,
}

impl Store {
    pub fn new(y_val: Operand, x_val: Operand) -> Self {
        let string = String::from("store ") + &y_val.get_value() + " " + &x_val.get_value();
        Store { y_val, x_val, p_command: string }
    }
}

impl Inst for Store {
    fn p_command(&self) -> &str {
        self.p_command.as_str()
    }
}

/// move ///

pub struct Move {
    y_val: Operand,
    x_val: Operand,
    p_command: String,
}

impl Move {
    pub fn new(y_val: Operand, x_val: Operand) -> Self {
        let string = String::from("move ") + &y_val.get_value() + " " + &x_val.get_value();
        Move { y_val, x_val, p_command: string }
    }
}

impl Inst for Move {
    fn p_command(&self) -> &str {
        self.p_command.as_str()
    }
}

/// phi ///

pub struct Phi {
    x_val: Vec<Operand>,
    p_command: String,
}

impl Phi {
    pub fn new(x_val: Vec<Operand>) -> Self {
        let mut string = String::from("phi := (");
        let mut first = true;
        for val in x_val.clone() {
            if !first { string += ", "; first = false; }
            string += &String::from(val.get_value());
        }
        string += ")";
        Phi { x_val, p_command: string }
    }
}

impl Inst for Phi {
    fn p_command(&self) -> &str {
        self.p_command.as_str()
    }
}

/// end ///

pub struct End {
    p_command: String,
}

impl End {
    pub fn new() -> Self {
        let string = String::from("end ");
        End { p_command: string }
    }
}

impl Inst for End {
    fn p_command(&self) -> &str {
        self.p_command.as_str()
    }
}

/// bra ///

pub struct Bra {
    y_val: Operand,
    p_command: String,
}

impl Bra {
    pub fn new(y_val: Operand) -> Self {
        let string = String::from("bra ") + &y_val.get_value();
        Bra { y_val, p_command: string }
    }
}

impl Inst for Bra {
    fn p_command(&self) -> &str {
        self.p_command.as_str()
    }
}

/// bne ///

pub struct BNE {
    x_val: Operand,
    y_val: Operand,
    p_command: String,
}

impl BNE {
    pub fn new(x_val: Operand, y_val: Operand) -> Self {
        let string = String::from("bne ") + &x_val.get_value() + " " + &y_val.get_value();
        BNE { x_val, y_val, p_command: string }
    }
}

impl Inst for BNE {
    fn p_command(&self) -> &str {
        self.p_command.as_str()
    }
}

/// beq ///

pub struct BEQ {
    x_val: Operand,
    y_val: Operand,
    p_command: String,
}

impl BEQ {
    pub fn new(x_val: Operand, y_val: Operand) -> Self {
        let string = String::from("beq ") + &x_val.get_value() + " " + &y_val.get_value();
        BEQ { x_val, y_val, p_command: string }
    }
}

impl Inst for BEQ {
    fn p_command(&self) -> &str {
        self.p_command.as_str()
    }
}

/// ble ///

pub struct BLE {
    x_val: Operand,
    y_val: Operand,
    p_command: String,
}

impl BLE {
    pub fn new(x_val: Operand, y_val: Operand) -> Self {
        let string = String::from("ble ") + &x_val.get_value() + " " + &y_val.get_value();
        BLE { x_val, y_val, p_command: string }
    }
}

impl Inst for BLE {
    fn p_command(&self) -> &str {
        self.p_command.as_str()
    }
}

/// blt ///

pub struct BLT {
    x_val: Operand,
    y_val: Operand,
    p_command: String,
}

impl BLT {
    pub fn new(x_val: Operand, y_val: Operand) -> Self {
        let string = String::from("blt ") + &x_val.get_value() + " " + &y_val.get_value();
        BLT { x_val, y_val, p_command: string }
    }
}

impl Inst for BLT {
    fn p_command(&self) -> &str {
        self.p_command.as_str()
    }
}

/// bge ///

pub struct BGE {
    x_val: Operand,
    y_val: Operand,
    p_command: String,
}

impl BGE {
    pub fn new(x_val: Operand, y_val: Operand) -> Self {
        let string = String::from("bge ") + &x_val.get_value() + " " + &y_val.get_value();
        BGE { x_val, y_val, p_command: string }
    }
}

impl Inst for BGE {
    fn p_command(&self) -> &str {
        self.p_command.as_str()
    }
}

/// bgt ///

pub struct BGT {
    x_val: Operand,
    y_val: Operand,
    p_command: String,
}

impl BGT {
    pub fn new(x_val: Operand, y_val: Operand) -> Self {
        let string = String::from("bgt ") + &x_val.get_value() + " " + &y_val.get_value();
        BGT { x_val, y_val, p_command: string }
    }
}

impl Inst for BGT {
    fn p_command(&self) -> &str {
        self.p_command.as_str()
    }
}

/// read ///

pub struct Read {
    p_command: String,
}

impl Read {
    pub fn new() -> Self {
        let string = String::from("read ");
        Read { p_command: string }
    }
}

impl Inst for Read {
    fn p_command(&self) -> &str {
        self.p_command.as_str()
    }
}

/// write ///

pub struct Write {
    x_val: Operand,
    p_command: String,
}

impl Write {
    pub fn new(x_val: Operand) -> Self {
        let string = String::from("write ") + &x_val.get_value();
        Write { x_val, p_command: string }
    }
}

impl Inst for Write {
    fn p_command(&self) -> &str {
        self.p_command.as_str()
    }
}

/// writenl ///

pub struct WriteNL {
    p_command: String,
}

impl WriteNL {
    pub fn new() -> Self {
        let string = String::from("writenl ");
        WriteNL { p_command: string }
    }
}

impl Inst for WriteNL {
    fn p_command(&self) -> &str {
        self.p_command.as_str()
    }
}

/// call ///

pub struct Call {
    p_command: String,
}

impl Call {
    pub fn new() -> Self {
        let string = String::from("call ");
        Call { p_command: string }
    }
}

impl Inst for Call {
    fn p_command(&self) -> &str {
        self.p_command.as_str()
    }
}

/// return ///

pub struct Return {
    x_val: Operand,
    p_command: String,
}

impl Return {
    pub fn new(x_val: Operand) -> Self {
        let string = String::from("return ") + &x_val.get_value();
        Return { x_val, p_command: string }
    }
}

impl Inst for Return {
    fn p_command(&self) -> &str {
        self.p_command.as_str()
    }
}