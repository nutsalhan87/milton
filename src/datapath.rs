use std::collections::VecDeque;

pub enum AluOperation {
    Sign,
    And,
    Or,
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    Left,
    Right,
}

impl AluOperation {
    fn op(&self, l: u32, r: u32) -> u32 {
        match self {
            AluOperation::Sign => r >> 31,
            AluOperation::And => l & r,
            AluOperation::Or => l | r,
            AluOperation::Add => l.wrapping_add(r),
            AluOperation::Sub => l.wrapping_sub(r),
            AluOperation::Mul => l.wrapping_mul(r),
            AluOperation::Div => l.wrapping_div(r),
            AluOperation::Rem => l.wrapping_rem(r),
            AluOperation::Left => l,
            AluOperation::Right => r,
        }
    }
}

impl Default for AluOperation {
    fn default() -> Self {
        AluOperation::Add
    }
}

pub struct DataPath {
    pub mem: [u8; 65536],
    pub acc: u32,
    pub sp: u16,
    pub input: VecDeque<u8>,
    pub output: Vec<u8>,
}

impl DataPath {
    pub fn new(mem: [u8; 65536]) -> Self {
        Self {
            mem,
            acc: 0,
            sp: u16::MAX - 4,
            input: VecDeque::new(),
            output: Vec::new(),
        }
    }

    pub fn process(
        &mut self,
        arg: u16,
        addr_mode: [bool; 2],
        alu_op: AluOperation,
        write: bool,
        latch_acc: bool,
        latch_stack: bool,
        extend_arg: bool
    ) -> (u32, bool) {
        let sum_sp_arg = self.sp.wrapping_add(arg);
        let data_addr = if addr_mode[0] { sum_sp_arg } else { arg };
        let data_read = self.load(data_addr);
        let arg_extended = arg as i16 as i32 as u32;
        let arg_selected = if extend_arg { arg_extended } else { arg as u32 };
        let mux_arg_acc = if addr_mode[0] { self.acc } else { arg_selected };
        let operand = if addr_mode[1] { mux_arg_acc } else { data_read };
        let res = alu_op.op(self.acc, operand);
        if latch_acc {
            self.acc = res;
        }
        if latch_stack {
            self.sp = self.sp.wrapping_add(res as i32 as i16 as u16);
        }
        if write {
            self.save(data_addr, self.acc);
        }

        (self.acc, self.acc == 0)
    }

    fn load(&mut self, addr: u16) -> u32 {
        if (0..4).contains(&addr) {
            self.input.pop_front().unwrap_or(0).into()
        } else if (4..8).contains(&addr) {
            0
        } else {
            let addr: usize = addr.into();
            u32::from_le_bytes([
                *self.mem.get(addr).unwrap_or(&0),
                *self.mem.get(addr + 1).unwrap_or(&0),
                *self.mem.get(addr + 2).unwrap_or(&0),
                *self.mem.get(addr + 3).unwrap_or(&0),
            ])
        }
    }

    fn save(&mut self, addr: u16, val: u32) {
        if (0..4).contains(&addr) {
            ()
        } else if (4..8).contains(&addr) {
            self.output.push(val as u8)
        } else {
            let addr: usize = addr.into();
            for (idx, byte) in val.to_le_bytes().into_iter().enumerate() {
                if addr + idx < u16::MAX as usize {
                    self.mem[addr + idx] = byte;
                }
            }
        }
    }
}
