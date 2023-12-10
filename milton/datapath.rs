use std::collections::VecDeque;

#[derive(Default)]
pub enum AluOperation {
    Sign,
    And,
    Or,
    #[default]
    Add,
    Sub,
    Mul,
    Div,
    Rem,
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
            AluOperation::Right => r,
        }
    }
}

pub struct DataPathSignals {
    pub arg: u16,
    pub addr_mode: [bool; 2],
    pub alu_op: AluOperation,
    pub write: bool,
    pub latch_acc: bool,
    pub latch_stack: bool,
    pub extend_arg: bool,
    pub io: bool,
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

    pub fn process(&mut self, signals: DataPathSignals) -> (u32, bool) {
        let sum_sp_arg = self.sp.wrapping_add(signals.arg);

        let data_addr = if signals.addr_mode[0] {
            sum_sp_arg
        } else {
            signals.arg
        };

        let data_read = self.load(data_addr, signals.io);

        let arg_extended = signals.arg as i16 as i32 as u32;

        let arg_selected = if signals.extend_arg {
            arg_extended
        } else {
            signals.arg as u32
        };

        let mux_arg_acc = if signals.addr_mode[0] {
            self.acc
        } else {
            arg_selected
        };

        let operand = if signals.addr_mode[1] {
            mux_arg_acc
        } else {
            data_read
        };

        let res = signals.alu_op.op(self.acc, operand);

        if signals.latch_acc {
            self.acc = res;
        }
        if signals.latch_stack {
            self.sp = self.sp.wrapping_add(res as i32 as i16 as u16);
        }
        if signals.write {
            self.save(data_addr, self.acc, signals.io);
        }

        (res, self.acc == 0)
    }

    fn load(&mut self, addr: u16, io: bool) -> u32 {
        if (0..4).contains(&addr) {
            if io {
                self.input.pop_front().unwrap_or(0).into()
            } else {
                0
            }
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

    fn save(&mut self, addr: u16, val: u32, io: bool) {
        if (4..8).contains(&addr) && io {
            self.output.push(val as u8)
        } else if !(0..4).contains(&addr) {
            let addr: usize = addr.into();
            for (idx, byte) in val.to_le_bytes().into_iter().enumerate() {
                if addr + idx < u16::MAX as usize {
                    self.mem[addr + idx] = byte;
                }
            }
        }
    }
}
