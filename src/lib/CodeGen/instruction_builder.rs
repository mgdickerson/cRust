// Generate each individual instruction here.
use std::rc::Rc;
use std::cell::RefCell;

use lib::IR::ir::{Op};

pub struct InstructionBuilder {
    id_counter: usize,
    inst_list: Vec<Instruction>,
}

impl InstructionBuilder {
    pub fn new() -> Self {
        InstructionBuilder { id_counter: 0, inst_list: Vec::new() }
    }

    // Will probably need more context to this.
    pub fn build_instruction(&mut self, op: Rc<RefCell<Op>>) {

    }
}

pub struct Instruction {
    inst: u32,
    id: usize,
}

struct InstructionPacker {}

impl InstructionPacker {
    fn pack_f1(opcode: OpCode, reg_a: u8, reg_b: u8, c_val: u16) -> u32 {
        let mut inst : u32 = 0;

        inst |= (opcode as u32) << 26;
        inst |= (reg_a as u32) << 21;
        inst |= (reg_b as u32) << 16;
        inst |= ((c_val as u32) & 0x0000FFFF); // Dont think this is necessary

        inst
    }

    fn pack_f2(opcode: OpCode, reg_a: u8, reg_b: u8, reg_c: u8) -> u32 {
        let mut inst : u32 = 0;

        inst |= (opcode as u32) << 26;
        inst |= (reg_a as u32) << 21;
        inst |= (reg_b as u32) << 16;
        inst |= reg_c as u32;

        inst
    }

    fn pack_f3(opcode: OpCode, c_val: u32) -> u32 {
        let mut inst : u32 = 0;

        inst |= (opcode as u32) << 26;
        inst |= (c_val & 0x03FFFFFF); // Masks the first 6 bits

        inst
    }
}

enum OpCode {
    // Arithmetic Instructions
    ADD = 0,
    SUB = 1,
    MUL = 2,
    DIV = 3,
    MOD = 4,
    CMP = 5,
    OR  = 8,
    AND = 9,
    BIC = 10,
    XOR = 11,

    LSH = 12,
    ASH = 13,

    CHK = 14,

    // Immediate Arithmetic Instructions
    ADDI = 16,
    SUBI = 17,
    MULI = 18,
    DIVI = 19,
    MODI = 20,
    CMPI = 21,
    ORI  = 24,
    ANDI = 25,
    BICI = 26,
    XORI = 27,

    LSHI = 28,
    ASHI = 29,

    CHKI = 30,

    // Load/Store Instructions
    LDW = 32,
    LDX = 33,
    POP = 34,
    STW = 36,
    STX = 37,
    PSH = 38,


    // Control Instructions
    BEQ = 40,
    BNE = 41,
    BLT = 42,
    BGE = 43,
    BLE = 44,
    BGT = 45,

    BSR = 46,
    JSR = 48,
    RET = 49,

    // Input/Output Instructions
    RDD = 50,
    WRD = 51,
    WRH = 52,
    WRL = 53,
}