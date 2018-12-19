// The DLX Virtual Machine
// chs / mf 2001-08-07

// All variables and methods are realized as class variables/methods which
// means that just one processor can be emulated at a time.
const MemSize: usize = 10000;

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
    M: [i32; MemSize/4],
}

impl DLX {
    pub fn new() -> Self {

    }

    pub fn load(&mut self, program: &mut [int]) {
        let mut i = 0;
        for OpCode in program {
            self.M[i] = OpCode;
            i+=1;
        }
        M[i] = -1;  // set first opcode of first instruction after program
        // to ERR in order to detect 'fall off the edge' errors
    }

    pub fn execute(&mut self) -> Result {
        let mut origc = 0;
        for i in 0..32 {
            self.R[i] = 0;
        }
        self.PC = 0;
        self.R[30] = (MemSize - 1) as i32;

        loop {
            self.R[0] = 0;
            disassem(self.M[self.PC])?; // initializes op, a, b, c

            let nextPC = self.PC + 1;
            if(self.format == 2) {
                origc = self.c;             // used for RET
                self.c = self.R[self.c];    // dirty trick
            }
            match self.op {
                ADD | ADDI => {
                    self.R[self.a] = self.R[self.b] + self.c;
                }
                SUB | SUBI => {
                    self.R[self.a] = self.R[self.b] + self.c;
                }
            }
        }
    }
}