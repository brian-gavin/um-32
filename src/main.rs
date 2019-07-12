use std::{u32, io::{self, prelude::*}};


#[derive(Debug)]
struct Cpu {
    regs: [u32; 8],
    pc: usize,
    halted: bool,
    memory: Vec<u32>,
}

struct Operator {
    data: u32,
}

impl From<u32> for Operator {
    fn from(n: u32) -> Operator {
        Operator {
            data: n,
        }
    }
}

impl Operator {
    pub fn A(&self) -> usize {
        ((self.data & 0700) >> 6) as u8 as usize
    }

    pub fn B(&self) -> usize {
        ((self.data & 070) >> 3) as u8 as usize
    }

    pub fn C(&self) -> usize {
        (self.data & 07) as u8 as usize
    }

    pub fn number(&self) -> usize {
        ((self.data & 0xf0000000) >> 28) as u8 as usize
    }
}

impl Cpu {
    pub fn new(program_scroll: &[u32]) -> Cpu {
        Cpu {
            regs: [0,0,0,0,0,0,0,0],
            pc: 0,
            halted: false,
            memory: Vec::from(program_scroll),
        }
    }

    pub fn spin_cycle(&mut self) {
        while !self.halted {
            let op = &Operator::from(self.memory[self.pc]);
            match op.number() {
                0 => self.conditional_move(op),
                1 => self.array_index(op),
                2 => self.array_amendment(op),
                3 => self.addition(op),
                4 => self.multiplication(op),
                5 => self.division(op),
                6 => self.not_and(op),
                7 => self.halt(),
                8 => self.allocation(op),
                _ => panic!("Unknown op number!"),
            }
        }
    }

    fn conditional_move(&mut self, op: &Operator) {
        if op.C() != 0 {
            self.regs[op.A()] = self.regs[op.B()]
        }
    }

    fn array_index(&mut self, op: &Operator) {
        let array_idx = op.B();
        let offset = op.C();
        self.regs[op.A()] = self.memory[array_idx + offset];
    }

    fn array_amendment(&mut self, op: &Operator) {
        let array_idx = op.A();
        let offset = op.B();
        self.memory[array_idx + offset] = self.regs[op.C()];
    }

    fn addition(&mut self, op: &Operator) {
        self.regs[op.A()] = (self.regs[op.B()] + self.regs[op.C()]) % u32::MAX;
    }

    fn multiplication(&mut self, op: &Operator) {
        self.regs[op.A()] = (self.regs[op.B()] * self.regs[op.C()]) % u32::MAX;
    }

    fn division(&mut self, op: &Operator) {
        self.regs[op.A()] = self.regs[op.B()] / self.regs[op.C()];
    }

    fn not_and(&mut self, op: &Operator) {
        self.regs[op.A()] = !(self.regs[op.B()] & self.regs[op.C()]);
    }

    fn halt(&mut self) {
        self.halted = true;
    }

    fn allocation(&mut self, _op: &Operator) {
        // todo
    }

    fn abandonment(&mut self, _op: &Operator) {
        // todo
    }

    fn output(&mut self, op: &Operator) {
        io::stdout().lock().write(&[self.regs[op.C()] as u8]).expect("Error writing to stdout");
    }

    fn input(&mut self, op: &Operator) {
        let mut c = [0u8; 1];
        io::stdin().lock().read_exact(&mut c).expect("Could not read from stdin");
        self.regs[op.C()] = c[0].into();
    }

    fn load_program(&mut self, op: &Operator) {
        let array_index = self.regs[op.B()] as usize;
        // for i in &self.memory[array_index..] {
            // self.memory[0] = *i
        // }
    }
}

fn main() {
    let mut cpu = Cpu::new(&[0]);
    println!("{:?}", cpu);
    cpu.spin_cycle();
}
