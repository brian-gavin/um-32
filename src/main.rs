use std::{
    collections::HashMap,
    io::{self, prelude::*},
    u32,
};

macro_rules! fail {
    ($($arg:tt)*) => {
        {
            eprint!("fail: ");
            eprintln!($($arg)*);
            std::process::exit(1)
        }
    };
}

#[derive(Debug)]
struct Cpu {
    regs: [u32; 8],
    pc: u32,
    halted: bool,
    memory: HashMap<u32, Box<[u32]>>,
    reuse: Vec<u32>,
}

struct Operator(u32);

#[allow(non_snake_case)]
impl Operator {
    pub fn A(&self) -> usize {
        ((self.0 & 0o700) >> 6) as _
    }

    pub fn B(&self) -> usize {
        ((self.0 & 0o70) >> 3) as _
    }

    pub fn C(&self) -> usize {
        (self.0 & 0o7) as _
    }

    pub fn number(&self) -> usize {
        ((self.0 & 0xf0000000) >> 28) as _
    }

    pub fn A_special(&self) -> usize {
        ((self.0 & 0x0e000000) >> 25) as _
    }

    pub fn value(&self) -> u32 {
        (self.0 & 0x00ffffff) as _
    }
}

impl Cpu {
    pub fn new(program_scroll: &[u32]) -> Cpu {
        Cpu {
            regs: [0, 0, 0, 0, 0, 0, 0, 0],
            pc: 0,
            halted: false,
            memory: {
                let mut m = HashMap::new();
                m.insert(0, Vec::from(program_scroll).into());
                m
            },
            reuse: vec![],
        }
    }

    pub fn spin_cycle(&mut self) {
        while !self.halted {
            let op = self.memory.get(&0).expect("no program scroll")[self.pc as usize];
            let op = Operator(op);
            let operation = match op.number() {
                0 => Self::conditional_move,
                1 => Self::array_index,
                2 => Self::array_amendment,
                3 => Self::addition,
                4 => Self::multiplication,
                5 => Self::division,
                6 => Self::not_and,
                7 => Self::halt,
                8 => Self::allocation,
                9 => Self::abandonment,
                10 => Self::output,
                11 => Self::input,
                12 => Self::load_program,
                13 => Self::orthography,
                n => fail!("unknown op number: {}", n),
            };
            operation(self, op);
        }
    }

    fn conditional_move(&mut self, op: Operator) {
        if self.regs[op.C()] != 0 {
            self.regs[op.A()] = self.regs[op.B()]
        }
    }

    fn array_index(&mut self, op: Operator) {
        let idx = self.regs[op.B()];
        let offset = self.regs[op.C()];
        self.regs[op.A()] = self.memory.get(&idx).expect("no array at index")[offset as usize];
    }

    fn array_amendment(&mut self, op: Operator) {
        let idx = self.regs[op.A()];
        let offset = self.regs[op.B()];
        self.memory.get_mut(&idx).expect("no array at index")[offset as usize] = self.regs[op.C()];
    }

    fn addition(&mut self, op: Operator) {
        self.regs[op.A()] = (self.regs[op.B()] + self.regs[op.C()]) % u32::MAX;
    }

    fn multiplication(&mut self, op: Operator) {
        self.regs[op.A()] = (self.regs[op.B()] * self.regs[op.C()]) % u32::MAX;
    }

    fn division(&mut self, op: Operator) {
        self.regs[op.A()] = self.regs[op.B()] / self.regs[op.C()];
    }

    fn not_and(&mut self, op: Operator) {
        self.regs[op.A()] = !(self.regs[op.B()] & self.regs[op.C()]);
    }

    fn halt(&mut self, _op: Operator) {
        self.halted = true;
    }

    fn allocation(&mut self, op: Operator) {
        let idx = if !self.reuse.is_empty() {
            self.reuse.pop().unwrap()
        } else {
            self.memory.len() as u32 + 1
        };
        let array = vec![0; self.regs[op.C()] as usize].into_boxed_slice();
        if self.memory.insert(idx, array).is_some() {
            panic!("BUG: index incorrectly calculated: {} was in use.", idx);
        }
    }

    fn abandonment(&mut self, op: Operator) {
        let idx = self.regs[op.C()];
        if idx == 0 {
            fail!("attempt to abandon the 0 array");
        }
        if self.memory.remove(&idx).is_none() {
            fail!("removing in-use index {}", idx)
        }
        self.reuse.push(idx);
    }

    fn output(&mut self, op: Operator) {
        io::stdout()
            .lock()
            .write(&[self.regs[op.C()] as u8])
            .expect("Error writing to stdout");
    }

    fn input(&mut self, op: Operator) {
        let mut c = [0u8; 1];
        io::stdin()
            .lock()
            .read_exact(&mut c)
            .expect("Could not read from stdin");
        self.regs[op.C()] = c[0].into();
    }

    fn load_program(&mut self, op: Operator) {
        let idx = self.regs[op.B()];
        let program = self
            .memory
            .get(&idx)
            .or_else(|| fail!("no array at index {}", idx))
            .unwrap()
            .clone();
        self.memory.insert(0, program);
    }

    fn orthography(&mut self, op: Operator) {
        self.regs[op.A_special()] = op.value();
    }
}

fn main() {
    let mut cpu = Cpu::new(&[0]);
    println!("{:?}", cpu);
    cpu.spin_cycle();
}

#[cfg(test)]
mod tests {
    use super::Operator;
    #[test]
    fn test_operator() {
        let op = Operator(0xe0000000 | 0o600 | 0o50 | 4);
        assert_eq!(
            op.0, 0xe00001ac,
            "expected: {:x} got: {:x}",
            0xe00001acu32, op.0
        );
        assert_eq!(op.number(), 0xe);
        assert_eq!(op.A(), 6);
        assert_eq!(op.B(), 5);
        assert_eq!(op.C(), 4);
    }
    #[test]
    fn test_operator_special() {
        let op = Operator(0xe0000000 | 0x0f000000 | 0xacab);
        assert_eq!(op.number(), 0xe, "op.number(): {:x}", op.number());
        assert_eq!(op.A_special(), 7, "op.A_special(): {:x}", op.A_special());
        assert_eq!(op.value(), 0xacab);
    }
}
