pub trait Inst {
    fn p_command(&self) -> &str;

    fn debugPrint(&self) {
        println!("{}", self.p_command());
    }
}

/// neg ///

pub struct Neg {
    x_val: i32,
    p_command: String,
}

impl Neg {
    pub fn new(x_val: i32) -> Self {
       Neg { x_val, p_command: String::from("neg ") + &x_val.to_string() }
    }
}

impl Inst for Neg {
    fn p_command(&self) -> &str {
        self.p_command.as_str()
    }
}

/// add ///

pub struct Add {
    x_val: i32,
    y_val: i32,
    p_command: String,
}

impl Add {
    pub fn new(x_val: i32, y_val: i32) -> Self {
        let string = String::from("add ") + &x_val.to_string() + " " + &y_val.to_string();
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
    x_val: i32,
    y_val: i32,
    p_command: String,
}

impl Sub {
    pub fn new(x_val: i32, y_val: i32) -> Self {
        let string = String::from("sub ") + &x_val.to_string() + " " + &y_val.to_string();
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
    x_val: i32,
    y_val: i32,
    p_command: String,
}

impl Mul {
    pub fn new(x_val: i32, y_val: i32) -> Self {
        let string = String::from("mul ") + &x_val.to_string() + " " + &y_val.to_string();
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
    x_val: i32,
    y_val: i32,
    p_command: String,
}

impl Div {
    pub fn new(x_val: i32, y_val: i32) -> Self {
        let string = String::from("div ") + &x_val.to_string() + " " + &y_val.to_string();
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
    x_val: i32,
    y_val: i32,
    p_command: String,
}

impl Cmp {
    pub fn new(x_val: i32, y_val: i32) -> Self {
        let string = String::from("cmp ") + &x_val.to_string() + " " + &y_val.to_string();
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
    x_val: i32,
    y_val: i32,
    p_command: String,
}

impl Adda {
    pub fn new(x_val: i32, y_val: i32) -> Self {
        let string = String::from("adda ") + &x_val.to_string() + " " + &y_val.to_string();
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
    y_val: i32,
    p_command: String,
}

impl Load {
    pub fn new(x_val: i32, y_val: i32) -> Self {
        let string = String::from("load ") + &y_val.to_string();
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
    y_val: i32,
    x_val: i32,
    p_command: String,
}

impl Store {
    pub fn new(y_val: i32, x_val: i32) -> Self {
        let string = String::from("store ") + &y_val.to_string() + " " + &x_val.to_string();
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
    y_val: i32,
    x_val: i32,
    p_command: String,
}

impl Move {
    pub fn new(y_val: i32, x_val: i32) -> Self {
        let string = String::from("move ") + &y_val.to_string() + " " + &x_val.to_string();
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
    x_val: Vec<i32>,
    p_command: String,
}

impl Phi {
    pub fn new(x_val: Vec<i32>) -> Self {
        let mut string = String::from("phi := (");
        let mut first = true;
        for val in x_val.clone() {
            if !first { string += ", "; first = false; }
            string += &String::from(val.to_string());
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
    y_val: i32,
    p_command: String,
}

impl Bra {
    pub fn new(y_val: i32) -> Self {
        let string = String::from("bra ") + &y_val.to_string();
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
    x_val: i32,
    y_val: i32,
    p_command: String,
}

impl BNE {
    pub fn new(x_val: i32, y_val: i32) -> Self {
        let string = String::from("bne ") + &x_val.to_string() + " " + &y_val.to_string();
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
    x_val: i32,
    y_val: i32,
    p_command: String,
}

impl BEQ {
    pub fn new(x_val: i32, y_val: i32) -> Self {
        let string = String::from("beq ") + &x_val.to_string() + " " + &y_val.to_string();
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
    x_val: i32,
    y_val: i32,
    p_command: String,
}

impl BLE {
    pub fn new(x_val: i32, y_val: i32) -> Self {
        let string = String::from("ble ") + &x_val.to_string() + " " + &y_val.to_string();
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
    x_val: i32,
    y_val: i32,
    p_command: String,
}

impl BLT {
    pub fn new(x_val: i32, y_val: i32) -> Self {
        let string = String::from("blt ") + &x_val.to_string() + " " + &y_val.to_string();
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
    x_val: i32,
    y_val: i32,
    p_command: String,
}

impl BGE {
    pub fn new(x_val: i32, y_val: i32) -> Self {
        let string = String::from("bge ") + &x_val.to_string() + " " + &y_val.to_string();
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
    x_val: i32,
    y_val: i32,
    p_command: String,
}

impl BGT {
    pub fn new(x_val: i32, y_val: i32) -> Self {
        let string = String::from("bgt ") + &x_val.to_string() + " " + &y_val.to_string();
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
    x_val: i32,
    p_command: String,
}

impl Write {
    pub fn new(x_val: i32) -> Self {
        let string = String::from("write ") + &x_val.to_string();
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
    x_val: i32,
    p_command: String,
}

impl Return {
    pub fn new(x_val: i32) -> Self {
        let string = String::from("return ") + &x_val.to_string();
        Return { x_val, p_command: string }
    }
}

impl Inst for Return {
    fn p_command(&self) -> &str {
        self.p_command.as_str()
    }
}