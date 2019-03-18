// Generate each individual instruction here.
use std::rc::Rc;
use std::cell::RefCell;

use lib::IR::ir::{Op, InstTy, ValTy};
use std::collections::HashMap;

pub struct InstructionBuilder {
    id_counter: usize,
    inst_list: Vec<Instruction>,
    reg_list: HashMap<usize, usize>,
    branch_correction: HashMap<usize, usize>,
}

impl InstructionBuilder {
    pub fn new(reg_list: HashMap<usize,usize>, branch_correction: HashMap<usize, usize>) -> Self {
        InstructionBuilder { id_counter: 0, inst_list: Vec::new(), reg_list, branch_correction }
    }

    // Will probably need more context to this.
    pub fn build_instruction(&mut self, op: Rc<RefCell<Op>>) {
        let op_type = op.borrow().inst_type().clone();
        match op_type {
            InstTy::neg |
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
            InstTy::adda => {},
            InstTy::store | InstTy::load => {},
            _ => {
                self.unpack_simple_ir(op.clone());
            },
        }
    }

    fn unpack_simple_ir(&mut self, op: Rc<RefCell<Op>>) -> u32 {
        let op_type = op.borrow().inst_type().clone();

        let mut x = 0;
        match op.borrow().clone_x_val() {
            Some(val) => {
                match val.get_value() {
                    ValTy::reg(reg) => {
                        x = reg.to_usize();
                    },
                    ValTy::op(op) => {
                        x = self.reg_list.get(&op.borrow().get_inst_num()).unwrap().clone();
                    },
                    ValTy::con(con) => {
                        panic!("x value should never be a constant.");
                    },
                    ValTy::adr(adr) => {
                        // TODO : Still not really sure what i want to do with this.
                        x = 0;
                    },
                    ValTy::node_id(id) => {
                        panic!("I dont believe x can have a node_id");
                    },
                    _ => {
                        panic!("Unexpected encounter in getting x value.");
                    },
                }
            },
            None => {
                x = 0;
            }
        }

        let mut y = 0;
        match op.borrow().clone_y_val() {
            Some(val) => {
                match val.get_value() {
                    ValTy::reg(reg) => {
                        y = reg.to_usize();
                    },
                    ValTy::op(op) => {
                        y = self.reg_list.get(&op.borrow().get_inst_num()).unwrap().clone();
                    },
                    ValTy::con(con) => {
                        if con.clone() > 65535 {
                            panic!("For now I am ignoring sizes above u16, and will take care of it later.");
                        }
                        y = con.clone() as usize;
                    },
                    ValTy::adr(adr) => {
                        // TODO : Still not really sure what i want to do with this.
                        y = 0;
                    },
                    ValTy::node_id(id) => {
                        y = self.branch_correction.get(&id.index()).unwrap().clone();
                    },
                    _ => {
                        panic!("Unexpected encounter in getting x value.");
                    },
                }
            },
            None => {
                y = 0;
            }
        }

        match op_type {
            /// Arithmetic ///
            InstTy::mov => {
                let a = x;
                let b = 0;
                let c = y;

                let b = 0; // Represents R0

                let inst = InstructionPacker::pack_f1(OpCode::ADD, a as u32, b, c as u32);
                self.inst_list.push(Instruction::new(inst, &self.id_counter));
                self.id_counter += 1;

                return 0
            },
            InstTy::add => {
                let a = self.reg_list.get(&op.borrow().get_inst_num()).unwrap().clone() as u32;
                let b = x as u32;
                let c = y as u32;

                if let ValTy::con(is_immediate) = op.borrow().clone_y_val().unwrap().get_value() {
                    let inst = InstructionPacker::pack_f1(OpCode::ADDI, a, b, c);
                    self.inst_list.push(Instruction::new(inst, &self.id_counter));
                    self.id_counter += 1;

                    return 0
                }

                let inst = InstructionPacker::pack_f2(OpCode::ADD, a, b, c);
                self.inst_list.push(Instruction::new(inst, &self.id_counter));
                self.id_counter += 1;

                return 0
            },
            InstTy::sub => {
                let a = self.reg_list.get(&op.borrow().get_inst_num()).unwrap().clone() as u32;
                let b = x as u32;
                let c = y as u32;

                let inst;
                if let ValTy::con(is_immediate) = op.borrow().clone_y_val().unwrap().get_value() {
                    inst = InstructionPacker::pack_f1(OpCode::SUBI, a, b, c);
                } else {
                    inst = InstructionPacker::pack_f2(OpCode::SUB, a, b, c);
                }

                self.inst_list.push(Instruction::new(inst, &self.id_counter));
                self.id_counter += 1;

                return 0
            },
            InstTy::mul => {
                let a = self.reg_list.get(&op.borrow().get_inst_num()).unwrap().clone() as u32;
                let b = x as u32;
                let c = y as u32;

                let inst;
                if let ValTy::con(is_immediate) = op.borrow().clone_y_val().unwrap().get_value() {
                    inst = InstructionPacker::pack_f1(OpCode::MULI, a, b, c);
                } else {
                    inst = InstructionPacker::pack_f2(OpCode::MUL, a, b, c);
                }

                self.inst_list.push(Instruction::new(inst, &self.id_counter));
                self.id_counter += 1;

                return 0

            },
            InstTy::div => {
                let a = self.reg_list.get(&op.borrow().get_inst_num()).unwrap().clone() as u32;
                let b = x as u32;
                let c = y as u32;

                let inst;
                if let ValTy::con(is_immediate) = op.borrow().clone_y_val().unwrap().get_value() {
                    inst = InstructionPacker::pack_f1(OpCode::DIVI, a, b, c);
                } else {
                    inst = InstructionPacker::pack_f2(OpCode::DIV, a, b, c);
                }

                self.inst_list.push(Instruction::new(inst, &self.id_counter));
                self.id_counter += 1;

                return 0
            },
            InstTy::cmp => {
                let a = self.reg_list.get(&op.borrow().get_inst_num()).unwrap().clone() as u32;
                let b = x as u32;
                let c = y as u32;

                let inst;
                if let ValTy::con(is_immediate) = op.borrow().clone_y_val().unwrap().get_value() {
                    inst = InstructionPacker::pack_f1(OpCode::CMPI, a, b, c);
                } else {
                    inst = InstructionPacker::pack_f2(OpCode::CMP, a, b, c);
                }

                self.inst_list.push(Instruction::new(inst, &self.id_counter));
                self.id_counter += 1;

                return 0
            },

//            /// Load/Store ///
//            InstTy::load => {
//                return (OpCode::LDX, FMT::F2)
//            },
//            InstTy::store => {
//                return (OpCode::STX, FMT::F2)
//            },

            /// Control ///
            InstTy::bne => {
                let a = x as u32;
                let b = 0;
                let c = y as u32;

                let inst = InstructionPacker::pack_f1(OpCode::BNE, a, b, c);
                self.inst_list.push(Instruction::new(inst, &self.id_counter));
                self.id_counter += 1;

                return 0
            },
            InstTy::beq => {
                let a = x as u32;
                let b = 0;
                let c = y as u32;

                let inst = InstructionPacker::pack_f1(OpCode::BEQ, a, b, c);
                self.inst_list.push(Instruction::new(inst, &self.id_counter));
                self.id_counter += 1;

                return 0
            },
            InstTy::ble => {
                let a = x as u32;
                let b = 0;
                let c = y as u32;

                let inst = InstructionPacker::pack_f1(OpCode::BLE, a, b, c);
                self.inst_list.push(Instruction::new(inst, &self.id_counter));
                self.id_counter += 1;

                return 0
            },
            InstTy::blt => {
                let a = x as u32;
                let b = 0;
                let c = y as u32;

                let inst = InstructionPacker::pack_f1(OpCode::BLT, a, b, c);
                self.inst_list.push(Instruction::new(inst, &self.id_counter));
                self.id_counter += 1;

                return 0
            },
            InstTy::bge => {
                let a = x as u32;
                let b = 0;
                let c = y as u32;

                let inst = InstructionPacker::pack_f1(OpCode::BGE, a, b, c);
                self.inst_list.push(Instruction::new(inst, &self.id_counter));
                self.id_counter += 1;

                return 0
            },
            InstTy::bgt => {
                let a = x as u32;
                let b = 0;
                let c = y as u32;

                let inst = InstructionPacker::pack_f1(OpCode::BGT, a, b, c);
                self.inst_list.push(Instruction::new(inst, &self.id_counter));
                self.id_counter += 1;

                return 0
            },
            InstTy::bra => {
                let a = x as u32;
                let b = 0;
                let c = y as u32;

                let inst = InstructionPacker::pack_f1(OpCode::BSR, a, b, c);
                self.inst_list.push(Instruction::new(inst, &self.id_counter));
                self.id_counter += 1;

                return 0
            },

            /// Input/Output ///
            InstTy::read => {
                let a = self.reg_list.get(&op.borrow().get_inst_num()).unwrap().clone();

                let inst = InstructionPacker::pack_f2(OpCode::RDD, a as u32, 0, 0);
                self.inst_list.push(Instruction::new(inst, &self.id_counter));
                self.id_counter += 1;

                return 0
            },
            InstTy::writeNL => {
                let inst = InstructionPacker::pack_f1(OpCode::WRL, 0, 0, 0);
                self.inst_list.push(Instruction::new(inst, &self.id_counter));
                self.id_counter += 1;

                return 0
            },
            InstTy::write => {
                let b = x as u32;

                let inst = InstructionPacker::pack_f2(OpCode::WRD, 0, b, 0);
                self.inst_list.push(Instruction::new(inst, &self.id_counter));
                self.id_counter += 1;

                return 0
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

impl Instruction {
    pub fn new(inst: u32, id: & usize) -> Self {
        Instruction { inst, id: id.clone() }
    }
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