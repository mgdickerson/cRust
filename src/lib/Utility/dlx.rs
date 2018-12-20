// The DLX Virtual Machine
// chs / mf 2001-08-07

// All variables and methods are realized as class variables/methods which
// means that just one processor can be emulated at a time.
use std::io;

pub struct DLX {
    // processor state variables
    R: [i32; 32],
    PC: i32,
    op: i32,
    a: i32,
    b: i32,
    c: i32,
    format: i32,

    // emulated memory
     // bytes in memory (divisible by 4)
    M: [i32; (MemSize/4) as usize],
}

impl DLX {
    pub fn new() -> Self {
        DLX{ R:[0; 32], PC: 0, op: 0, a: 0, b: 0, c: 0, format: 0, M:[0; (MemSize/4) as usize] }
    }

    pub fn load(&mut self, program: &mut [i32]) {
        let mut i = 0;
        for OpCode in program {
            self.M[i] = *OpCode;
            i+=1;
        }
        self.M[i] = -1;  // set first opcode of first instruction after program
        // to ERR in order to detect 'fall off the edge' errors
    }

    // TODO : Add return type for error handling
    pub fn execute(&mut self) {
        let mut origc = 0;
        for i in 0..32 {
            self.R[i] = 0;
        }
        self.PC = 0;
        self.R[30] = (MemSize - 1) as i32;

        loop {
            self.R[0] = 0;
            let instructionWord : i32 = self.M[self.PC as usize];
            self.disassem(instructionWord); // initializes op, a, b, c

            let mut nextPC = self.PC + 1;
            if(self.format == 2) {
                origc = self.c;             // used for RET
                self.c = self.R[self.c as usize];    // dirty trick
            }
            match self.op {
                ADD | ADDI => {
                    self.R[self.a as usize] = self.R[self.b as usize] + self.c;
                }
                SUB | SUBI => {
                    self.R[self.a as usize] = self.R[self.b as usize] - self.c;
                }
                CMP | CMPI => {
                    self.R[self.a as usize] = self.R[self.b as usize] - self.c; // can not create overflow
                    if self.R[self.a as usize] < 0 {
                        self.R[self.a as usize] = -1;
                    }
                    else if self.R[self.a as usize] > 0 {
                        self.R[self.a as usize] = 1;
                    }
                    // do nothing if R[a] == 0;
                }
                MUL | MULI => {
                    self.R[self.a as usize] = self.R[self.b as usize] * self.c;
                }
                DIV | DIVI => {
                    self.R[self.a as usize] = self.R[self.b as usize] / self.c;
                }
                MOD | MODI => {
                    self.R[self.a as usize] = self.R[self.b as usize] % self.c;
                }
                OR | ORI => {
                    self.R[self.a as usize] = self.R[self.b as usize] | self.c;
                }
                AND | ANDI => {
                    self.R[self.a as usize] = self.R[self.b as usize] & self.c;
                }
                BIC | BICI => {
                    self.R[self.a as usize] = self.R[self.b as usize] & !self.c;
                }
                XOR | XORI => {
                    self.R[self.a as usize] = self.R[self.b as usize] ^ self.c;
                }
                LSH | LSHI => {
                    if (self.c < -31) || (self.c > 31) {
                        println!("Illegal value {} of operand c or register c!", self.c);
                        self.bug(1);
                    }
                    if self.c < 0 { self.R[self.a as usize] = (self.R[self.b as usize] as usize >> -self.c as usize) as i32 }
                    else {self.R[self.a as usize] = self.R[self.b as usize] << self.c }
                }
                ASH | ASHI => {
                    if (self.c < -31) || (self.c > 31) {
                        println!("DLX.Execute: Illegal value {} of operand c or register c!", self.c);
                        self.bug(1);
                    }
                    if self.c < 0 { self.R[self.a as usize] = self.R[self.b as usize] >> -self.c }
                        else {self.R[self.a as usize] = self.R[self.b as usize] << self.c }
                }
                CHK | CHKI => {
                    if self.R[self.a as usize] < 0 {
                        println!("DLX.Execute: {}: R[{}] == {} < 0", self.PC*4, self.a, self.R[self.a as usize]);
                        self.bug(14);
                    } else if self.R[self.a as usize] >= self.c {
                        println!("DLX.Execute: {}: R[{}] == {} >= {}", self.PC*4, self.a, self.R[self.a as usize], self.c);
                        self.bug(14);
                    }
                }
                LDW | LDX => {
                    // remember: c == R[origc] because of F2 format
                    self.R[self.a as usize] = self.M[((self.R[self.b as usize] + self.c) / 4) as usize];
                }
                STW | STX => {
                    // remember: c == R[origc] because of F2 format
                    self.M[((self.R[self.b as usize] + self.c) / 4) as usize] = self.R[self.a as usize];
                }
                POP => {
                    self.R[self.a as usize] = self.M[(self.R[self.b as usize] / 4) as usize];
                    self.R[self.b as usize] = self.R[self.b as usize] + self.c;
                }
                PSH => {
                    self.R[self.b as usize] = self.R[self.b as usize] + self.c;
                    self.M[(self.R[self.b as usize] / 4) as usize] = self.R[self.a as usize];
                }
                BEQ => {
                    if self.R[self.a as usize] == 0 { nextPC = self.PC + self.c; }
                    if (nextPC < 0) || (nextPC > MemSize/4) {
                        println!("{} is no address in memory (0..{}).", 4*nextPC, MemSize);
                        self.bug(40);
                    }
                }
                BNE => {
                    if self.R[self.a as usize] != 0 { nextPC = self.PC + self.c; }
                    if (nextPC < 0) || (nextPC > MemSize/4) {
                        println!("{} is no address in memory (0..{}).", 4*nextPC, MemSize);
                        self.bug(41);
                    }
                }
                BLT => {
                    if self.R[self.a as usize] < 0 { nextPC = self.PC + self.c; }
                    if (nextPC < 0) || (nextPC > MemSize/4) {
                        println!("{} is no address in memory (0..{}).", 4*nextPC, MemSize);
                        self.bug(42);
                    }
                }
                BGE => {
                    if self.R[self.a as usize] >= 0 { nextPC = self.PC + self.c; }
                    if (nextPC < 0) || (nextPC > MemSize/4) {
                        println!("{} is no address in memory (0..{}).", 4*nextPC, MemSize);
                        self.bug(42);
                    }
                }
                BLE => {
                    if self.R[self.a as usize] <= 0 { nextPC = self.PC + self.c; }
                    if (nextPC < 0) || (nextPC > MemSize/4) {
                        println!("{} is no address in memory (0..{}).", 4*nextPC, MemSize);
                        self.bug(42);
                    }
                }
                BGT => {
                    if self.R[self.a as usize] > 0 { nextPC = self.PC + self.c; }
                    if (nextPC < 0) || (nextPC > MemSize/4) {
                        println!("{} is no address in memory (0..{}).", 4*nextPC, MemSize);
                        self.bug(42);
                    }
                }
                BSR => {
                    self.R[31] = (self.PC + 1) * 4;
                    nextPC = self.PC + self.c;
                }
                JSR => {
                    self.R[31] = (self.PC + 1) * 4;
                    nextPC = self.c / 4;
                }
                RET => {
                    if origc == 0 { return; } // remember: c==R[origc]
                    if (self.c < 0) || (self.c > MemSize) {
                        println!("{} is no address in memory (0..{}).", self.c, MemSize);
                        self.bug(49);
                    }
                    nextPC = self.c / 4;
                }
                RDI => {
                    println!("?:");
                    let mut input = String::new();
                    io::stdin().read_line(&mut input);
                    self.R[self.a as usize] = input.parse().unwrap();
                }
                WRD => {
                    println!("{}  ", self.R[self.b as usize]);
                }
                WRH => {
                    println!("0x{:x}  ", self.R[self.b as usize]);
                }
                WRL => {
                    println!();
                }
                ERR => {
                    println!("Program dropped off the end!");
                    self.bug(1);
                }
                _ => {
                    println!("DLX.Execute: Unknown opcode encountered!");
                    self.bug(1);
                }
            }

            self.PC = nextPC;
        }
    }

    pub fn disassem(&mut self, instructionWord: i32) {
        self.op = instructionWord >> 26 as usize;    // without sign extension
        match self.op {
            // F1 Format
            BSR | RDI | WRD | WRH | WRL | CHKI | BEQ |
            BNE | BLT | BGE | BLE | BGT | ADDI | SUBI |
            MULI | DIVI | MODI | CMPI | ORI | ANDI | BICI |
            XORI | LSHI | ASHI | LDW | POP | STW | PSH => {
                self.format = 1;
                self.a = (instructionWord >> 21 as usize) & 0x1F;
                self.b = (instructionWord >> 16 as usize) & 0x1F;
                self.c = (instructionWord as u8) as i32;
            }
            // F2 Format
             RET | CHK | ADD | SUB | MUL |
             DIV | MOD | CMP | OR  | AND |
             BIC | XOR | LSH | ASH | LDX |
             STX => {
                 self.format = 2;
                 self.a = (instructionWord >> 21 as usize) & 0x1F;
                 self.b = (instructionWord >> 16 as usize) & 0x1F;
                 self.c = instructionWord & 0x1F;
             }
            // F3 Format
            JSR => {
                self.format = 3;
                self.a = -1;
                self.b = -1;
                self.c = instructionWord & 0x3FFFFFF;
            }
            _ => {
                println!("Illegal instruction! ({})", self.PC);
            }
        }
    }

    pub fn bug(&mut self, errNum: u32) {
        println!("Bug number: {}", errNum);
    }
}

// Const Keywords
const MemSize: i32 = 10000;
const ADD : i32 = 0;
const SUB : i32 = 1;
const MUL : i32 = 2;
const DIV : i32 = 3;
const MOD : i32 = 4;
const CMP : i32 = 5;
const OR  : i32 = 8;
const AND : i32 = 9;
const BIC : i32 = 10;
const XOR : i32 = 11;
const LSH : i32 = 12;
const ASH : i32 = 13;
const CHK : i32 = 14;

const ADDI : i32 = 16;
const SUBI : i32 = 17;
const MULI : i32 = 18;
const DIVI : i32 = 19;
const MODI : i32 = 20;
const CMPI : i32 = 21;
const ORI  : i32 = 24;
const ANDI : i32 = 25;
const BICI : i32 = 26;
const XORI : i32 = 27;
const LSHI : i32 = 28;
const ASHI : i32 = 29;
const CHKI : i32 = 30;

const LDW : i32 = 32;
const LDX : i32 = 33;
const POP : i32 = 34;
const STW : i32 = 36;
const STX : i32 = 37;
const PSH : i32 = 38;

const BEQ : i32 = 40;
const BNE : i32 = 41;
const BLT : i32 = 42;
const BGE : i32 = 43;
const BLE : i32 = 44;
const BGT : i32 = 45;
const BSR : i32 = 46;
const JSR : i32 = 48;
const RET : i32 = 49;

const RDI : i32 = 50;
const WRD : i32 = 51;
const WRH : i32 = 52;
const WRL : i32 = 53;

const ERR : i32 = 63;   // error opcode which is insertered by loader
// after end of program code
