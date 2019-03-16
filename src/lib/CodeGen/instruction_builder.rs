// Generate each individual instruction here.
use std::rc::Rc;
use std::cell::RefCell;

use lib::IR::ir::{Op, InstTy, ValTy};

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
        let op_type = op.borrow().inst_type().clone();
        match op_type {
            InstTy::neg | InstTy::mov |
            InstTy::end => {
                // Dont think this was used anywhere...
                panic!("Encountered instruction expected to not be used.");
            },
            InstTy::phi | InstTy::kill => {
                // These should all be removed by this point
                panic!("Should be no remaining phi instructions when in codegen.");
            },
            InstTy::spill | InstTy::loadsp |
            InstTy::pload | InstTy::gload |
            InstTy::pload | InstTy::gstore |
            InstTy::ret | InstTy::call => {},
            _ => {
                let unpacked_inst = InstructionBuilder::unpack_simple_ir(op.clone());
            },
        }
    }

    fn unpack_simple_ir(op: Rc<RefCell<Op>>) -> (OpCode, FMT) {
        let op_type = op.borrow().inst_type().clone();

        match op_type {
            /// Arithmetic ///
            InstTy::add | InstTy::adda => {
                if let ValTy::con(is_immediate) = op.borrow().clone_y_val().unwrap().get_value() {
                    return (OpCode::ADDI, FMT::F1)
                }

                return (OpCode::ADD, FMT::F2)
            },
            InstTy::sub => {
                if let ValTy::con(is_immediate) = op.borrow().clone_y_val().unwrap().get_value() {
                    return (OpCode::SUBI, FMT::F1)
                }

                return (OpCode::SUB, FMT::F2)
            },
            InstTy::mul => {
                if let ValTy::con(is_immediate) = op.borrow().clone_y_val().unwrap().get_value() {
                    return (OpCode::MULI, FMT::F1)
                }

                return (OpCode::MUL, FMT::F2)

            },
            InstTy::div => {
                if let ValTy::con(is_immediate) = op.borrow().clone_y_val().unwrap().get_value() {
                    return  (OpCode::DIVI, FMT::F1)
                }

                return (OpCode::DIV, FMT::F2)
            },
            InstTy::cmp => {
                if let ValTy::con(is_immediate) = op.borrow().clone_y_val().unwrap().get_value() {
                    return  (OpCode::CMPI, FMT::F1)
                }

                return (OpCode::CMP, FMT::F2)
            },

            /// Load/Store ///
            InstTy::load => {
                return (OpCode::LDX, FMT::F2)
            },
            InstTy::store => {
                return (OpCode::STX, FMT::F2)
            },

            /// Control ///
            InstTy::bne => {
                return (OpCode::BNE, FMT::F1)
            },
            InstTy::beq => {
                return (OpCode::BEQ, FMT::F1)
            },
            InstTy::ble => {
                return (OpCode::BLE, FMT::F1)
            },
            InstTy::blt => {
                return (OpCode::BLT, FMT::F1)
            },
            InstTy::bge => {
                return (OpCode::BGE, FMT::F1)
            },
            InstTy::bgt => {
                return (OpCode::BGT, FMT::F1)
            },
            InstTy::bra => {
                return (OpCode::BSR, FMT::F1)
            },

            /// Input/Output ///
            InstTy::read => {
                return (OpCode::RDD, FMT::F2)
            },
            InstTy::writeNL => {
                return (OpCode::WRL, FMT::F1)
            },
            InstTy::write => {
                return (OpCode::WRD, FMT::F2)
            },
            _ => {
                panic!("Found unexpected instruction while unpacking simple IR.");
            },
        }
    }

    fn unpack_complex_ir(op: Rc<RefCell<Op>>) {

    }
}

pub struct Instruction {
    inst: u32,
    id: usize,
}

struct InstructionPacker {}

impl InstructionPacker {
    fn pack_f1(opcode: OpCode, reg_a: u32, reg_b: u32, c_val: u32) -> u32 {
        let mut inst : u32 = 0;

        inst |= (opcode as u32) << 26;
        inst |= (reg_a) << 21;
        inst |= (reg_b) << 16;
        inst |= ((c_val) & 0x0000FFFF); // Dont think this is necessary

        inst
    }

    fn pack_f2(opcode: OpCode, reg_a: u32, reg_b: u32, reg_c: u32) -> u32 {
        let mut inst : u32 = 0;

        inst |= (opcode as u32) << 26;
        inst |= (reg_a) << 21;
        inst |= (reg_b) << 16;
        inst |= reg_c;

        inst
    }

    fn pack_f3(opcode: OpCode, c_val: u32) -> u32 {
        let mut inst : u32 = 0;

        inst |= (opcode as u32) << 26;
        inst |= (c_val & 0x03FFFFFF); // Masks the first 6 bits

        inst
    }
}

enum FMT {
    F1,
    F2,
    F3,
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