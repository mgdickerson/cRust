// Generate each individual instruction here.
use std::rc::Rc;
use std::cell::RefCell;

use lib::IR::ir::{Op, InstTy, ValTy};
use std::collections::HashMap;
use lib::RegisterAllocator::RegisterAllocation;

pub struct InstructionBuilder {
    id_counter: usize,
    inst_list: Vec<Instruction>,
    latest_adda: Option<Rc<RefCell<Op>>>,
    inst_position_tracker: HashMap<usize, usize>,
    branch_revisits: HashMap<usize, usize>,
}

impl InstructionBuilder {
    pub fn new() -> Self {
        let mut inst_list = Vec::new();
        let mut id_counter = 0;

        // Make Space to load location of Global Address Pointer
        let inst = InstructionPacker::pack_f1(OpCode::ADD, 0, 0, 0);
        inst_list.push(Instruction::new(inst, &id_counter));
        id_counter += 1;

        // Make Space to load current SP
        let inst = InstructionPacker::pack_f1(OpCode::ADD, 0, 0, 0);
        inst_list.push(Instruction::new(inst, &id_counter));
        id_counter += 1;

        InstructionBuilder {
            id_counter,
            inst_list,
            latest_adda: None,
            inst_position_tracker: HashMap::new(),
            branch_revisits: HashMap::new(),
        }
    }

    pub fn get_inst_list(&self) -> Vec<Instruction> {
        self.inst_list.clone()
    }

    pub fn patch_branches(&mut self) {
        for (instruction_id, op_id) in self.branch_revisits.clone() {
            let branch_position = self.inst_position_tracker.get(&op_id).unwrap().clone();

            self.inst_list.get_mut(instruction_id).unwrap().patch_f1_branch(branch_position as u32);
        }
    }

    pub fn patch_global_stack(&mut self, global_offset: u32, stack_offset: u32) {
        self.inst_list
            .get_mut(0)
            .unwrap()
            .alter_instruction(
                OpCode::ADDI,
                FMT::F1,
                RegisterAllocation::allocate_R30().to_u32(),
                0,
                global_offset
            );

        self.inst_list
            .get_mut(1)
            .unwrap()
            .alter_instruction(
                OpCode::ADDI,
                FMT::F1,
                RegisterAllocation::allocate_R29().to_u32(),
                0,
                stack_offset
            );
    }

    // Will probably need more context to this.
    pub fn build_instruction(&mut self, op: Rc<RefCell<Op>>, is_global: bool) {
        let inst_id = op.borrow().get_inst_num();
        self.inst_position_tracker.insert(inst_id, self.id_counter.clone());

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
            InstTy::spill | InstTy::loadsp => {
                // As of right now, the only spills are in the main,
                // but it is not currently handled if the spill happens in
                // a function or in a global.
                self.unpack_spill_loadsp(op, is_global);
            }
            // These are all function related, save this for absolute last.
            InstTy::pload | InstTy::gload |
            InstTy::pload | InstTy::gstore |
            InstTy::call => {
                // TODO : If I find myself with an excess of time tomorrow morning.
            },
            InstTy::adda => {
                self.latest_adda = Some(op);
            },
            InstTy::store | InstTy::load => {
                self.unpack_load_store(op);
            },
            _ => {
                self.unpack_simple_ir(op);
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
                        x = reg.to_u32();
                    },
                    ValTy::op(op) => {
                        // for now just make it 0, handle this later.
                        x = 0;
                    },
                    ValTy::con(con) => {
                        panic!("x value should never be a constant.");
                    },
                    ValTy::adr(adr) => {
                        // TODO : Still not really sure what i want to do with this.
                        panic!("Addresses should all be handled now.");
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
                        y = reg.to_u32() as i32;
                    },
                    ValTy::op(op) => {
                        y = op.borrow().get_inst_num() as i32;
                    },
                    ValTy::con(con) => {
                        if con.clone() > 65535 {
                            panic!("For now I am ignoring sizes above u16, and will take care of it later.");
                        }
                        y = con.clone();
                    },
                    ValTy::adr(adr) => {
                        // TODO : Still not really sure what i want to do with this.
                        y = 0;
                    },
                    ValTy::node_id(id) => {
                       panic!("NodeIds should have all be handled by code cleanup already.");
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
                let a = op.borrow().get_register().to_u32() as u32;
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
                let a = op.borrow().get_register().to_u32() as u32;
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
                let a = op.borrow().get_register().to_u32() as u32;
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
                let a = op.borrow().get_register().to_u32() as u32;
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
                let a = op.borrow().get_register().to_u32() as u32;
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

            /// Control ///
            InstTy::bne => {
                let a = x as u32;
                let b = 0;

                let inst = InstructionPacker::pack_f1(OpCode::BNE, a, b, 0);
                self.inst_list.push(Instruction::new(inst, &self.id_counter));
                self.branch_revisits.insert(self.id_counter.clone(), y as usize);
                self.id_counter += 1;

                return 0
            },
            InstTy::beq => {
                let a = x as u32;
                let b = 0;

                let inst = InstructionPacker::pack_f1(OpCode::BEQ, a, b, 0);
                self.inst_list.push(Instruction::new(inst, &self.id_counter));
                self.branch_revisits.insert(self.id_counter.clone(), y as usize);
                self.id_counter += 1;

                return 0
            },
            InstTy::ble => {
                let a = x as u32;
                let b = 0;

                let inst = InstructionPacker::pack_f1(OpCode::BLE, a, b, 0);
                self.inst_list.push(Instruction::new(inst, &self.id_counter));
                self.branch_revisits.insert(self.id_counter.clone(), y as usize);
                self.id_counter += 1;

                return 0
            },
            InstTy::blt => {
                let a = x as u32;
                let b = 0;

                let inst = InstructionPacker::pack_f1(OpCode::BLT, a, b, 0);
                self.inst_list.push(Instruction::new(inst, &self.id_counter));
                self.branch_revisits.insert(self.id_counter.clone(), y as usize);
                self.id_counter += 1;

                return 0
            },
            InstTy::bge => {
                let a = x as u32;
                let b = 0;

                let inst = InstructionPacker::pack_f1(OpCode::BGE, a, b, 0);
                self.inst_list.push(Instruction::new(inst, &self.id_counter));
                self.branch_revisits.insert(self.id_counter.clone(), y as usize);
                self.id_counter += 1;

                return 0
            },
            InstTy::bgt => {
                let a = x as u32;
                let b = 0;

                let inst = InstructionPacker::pack_f1(OpCode::BGT, a, b, 0);
                self.inst_list.push(Instruction::new(inst, &self.id_counter));
                self.branch_revisits.insert(self.id_counter.clone(), y as usize);
                self.id_counter += 1;

                return 0
            },
            InstTy::bra => {
                let a = 0;
                let b = 0;

                let inst = InstructionPacker::pack_f1(OpCode::BEQ, a, b, 0);
                self.inst_list.push(Instruction::new(inst, &self.id_counter));
                self.branch_revisits.insert(self.id_counter.clone(), y as usize);
                self.id_counter += 1;

                return 0
            },

            /// Input/Output ///
            InstTy::read => {
                let a = op.borrow().get_register().to_u32();

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

            InstTy::ret => {
                let c = x as u32;

                let inst = InstructionPacker::pack_f2(OpCode::RET, 0, 0, c);
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

    fn unpack_load_store(&mut self, op: Rc<RefCell<Op>>) {
        let op_type = op.borrow().inst_type().clone();

        let adda = self.latest_adda.clone().unwrap();
        let (x_val, y_val) = adda.borrow().get_val_ty();

        let mut x = 0;
        if let ValTy::reg(reg) = x_val
            .unwrap() {
            x = reg.to_u32();
        } else {
            panic!("Invalid register value in adda instruction.");
        }

        let mut y = 0;
        if let ValTy::reg(reg) = y_val
            .unwrap() {
            y = reg.to_u32();
        } else {
            panic!("Invalid register value found in adda instruction.");
        }


        let b = x;
        let c = y;

        match op_type {
            InstTy::store => {
                let mut a = 0;
                match op
                    .borrow()
                    .clone_y_val().unwrap()
                    .get_value()
                    .clone() {
                    ValTy::reg(reg) => {
                        a = reg.to_u32();
                    },
                    ValTy::con(con) => {
                        a = con as u32;
                    }
                    _ => {
                        panic!("Store instruction did not contain an appropriate value to store...");
                    }
                }
                let inst = InstructionPacker::pack_f2(OpCode::STX, a, b, c);
                self.inst_list.push(Instruction::new(inst, &self.id_counter));
                self.id_counter += 1;
            },
            InstTy::load => {
                let a = op.borrow().get_register().to_u32();
                let inst = InstructionPacker::pack_f2(OpCode::LDX, a, b, c);
                self.inst_list.push(Instruction::new(inst, &self.id_counter));
                self.id_counter += 1;
            },
            _ => {},
        }

        self.latest_adda = None;
    }

    fn unpack_spill_loadsp(&mut self, op: Rc<RefCell<Op>>, is_global: bool) {
        let op_type = op.borrow().inst_type().clone();

        match op_type {
            InstTy::spill => {
                let mut a = 0;
                if let ValTy::reg(spill_reg) = op
                    .borrow()
                    .clone_y_val()
                    .unwrap()
                    .get_value()
                    .clone() {
                    a = spill_reg.to_u32();
                } else {
                    panic!("Spill value was not a register.");
                }

                let mut b = 0;
                if is_global {
                    // Global Register
                    b = RegisterAllocation::allocate_R30().to_u32();
                } else {
                    // Stack Pointer
                    b = RegisterAllocation::allocate_R29().to_u32();
                }

                let mut c = 0;
                if let ValTy::con(addr_num) = op
                    .borrow()
                    .clone_x_val()
                    .unwrap()
                    .get_value()
                    .clone() {
                    c = addr_num as u32;
                } else {
                    panic!("Spill value was not a constant offset.");
                }

                let inst = InstructionPacker::pack_f1(OpCode::STW, a, b, c);
                self.inst_list.push(Instruction::new(inst, &self.id_counter));
                self.id_counter += 1;
            },
            InstTy::loadsp => {
                let mut a = op.borrow().get_register().to_u32();

                let mut b = 0;
                if is_global {
                    // Global Register
                    b = RegisterAllocation::allocate_R30().to_u32();
                } else {
                    // Stack Pointer
                    b = RegisterAllocation::allocate_R29().to_u32();
                }

                let mut c = 0;
                if let ValTy::con(addr_num) = op
                    .borrow()
                    .clone_y_val()
                    .unwrap()
                    .get_value()
                    .clone() {
                    c = addr_num as u32;
                } else {
                    panic!("Spill value was not a constant offset.");
                }

                let inst = InstructionPacker::pack_f1(OpCode::LDW, a, b, c);
                self.inst_list.push(Instruction::new(inst, &self.id_counter));
                self.id_counter += 1;
            },
            _ => {},
        }
    }
}

#[derive(Clone)]
pub struct Instruction {
    inst: u32,
    id: usize,
}

impl Instruction {
    pub fn new(inst: u32, id: & usize) -> Self {
        Instruction { inst, id: id.clone() }
    }

    pub fn get_inst(&self) -> u32 {
        self.inst.clone()
    }

    fn patch_f1_branch(&mut self, c: u32) {
        self.inst |= c;
    }

    fn alter_instruction(&mut self, opcode: OpCode, fmt: FMT, a: u32, b: u32, c: u32) {
        match fmt {
            FMT::F1 => {
                self.inst |= InstructionPacker::pack_f1(opcode, a, b, c);
            },
            FMT::F2 => {
                self.inst |= InstructionPacker::pack_f2(opcode, a, b, c);
            },
            FMT::F3 => {
                self.inst |= InstructionPacker::pack_f3(opcode, c);
            },
        }
    }
}

impl std::fmt::Debug for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}: {:032b}", self.id, self.inst)
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

#[derive(Clone)]
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