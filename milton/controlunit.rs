use crate::datapath::{AluOperation, DataPath, DataPathSignals};

#[derive(Default)]
struct InstructionDecoderResult {
    arg: u16,
    alu_op: AluOperation,
    addr_mode: [bool; 2],
    latch_acc: bool,
    latch_stack: bool,
    write: bool,
    extend_arg: bool,
    latch_ip: bool,
    if_zero: bool,
    jmp: bool,
    halt: bool,
    abs_jump: bool,
    io: bool,
}

struct InstructionDecoder {
    steps: u8,
}

impl InstructionDecoder {
    fn process(&mut self, word: u32, ip: u16, temp_reg: u32) -> InstructionDecoderResult {
        let instr = (word >> 24) as u8;
        let mut res = InstructionDecoderResult {
            arg: word as u16,
            addr_mode: [(word >> 22) % 2 == 1, (word >> 23) % 2 == 1],
            latch_ip: true,
            extend_arg: true,
            ..Default::default()
        };

        if (0x00..0x08).contains(&instr) {
            res.latch_acc = true;
            match instr {
                0x00 => {
                    res.alu_op = AluOperation::Sign;
                }
                0x01 => {
                    res.alu_op = AluOperation::And;
                    res.extend_arg = false;
                }
                0x02 => {
                    res.alu_op = AluOperation::Or;
                    res.extend_arg = false;
                }
                0x03 => {
                    res.alu_op = AluOperation::Add;
                }
                0x04 => {
                    res.alu_op = AluOperation::Sub;
                }
                0x05 => {
                    res.alu_op = AluOperation::Mul;
                }
                0x06 => {
                    res.alu_op = AluOperation::Div;
                }
                0x07 => {
                    res.alu_op = AluOperation::Rem;
                }
                _ => (),
            }
        } else if (0x08..0x0A).contains(&instr) {
            res.jmp = true;
            if instr == 0x09 {
                res.if_zero = true;
            }
        } else if instr == 0x0A {
            res.latch_ip = false;
            res.alu_op = AluOperation::Right;
            if self.steps == 0 {
                self.steps = 3;
                res.arg = -4i16 as u16;
                res.addr_mode = [false, true];
                res.latch_stack = true;
            } else if self.steps == 3 {
                self.steps -= 1;
                res.arg = ip.wrapping_add(1);
                res.addr_mode = [false, true];
                res.latch_acc = true;
            } else if self.steps == 2 {
                self.steps -= 1;
                res.arg = 0;
                res.addr_mode = [true, false];
                res.write = true;
            } else if self.steps == 1 {
                self.steps -= 1;
                res.addr_mode = [false, true];
                res.latch_ip = true;
                res.abs_jump = true;
            }
        } else if instr == 0x0B {
            res.alu_op = AluOperation::Right;
            if self.steps == 0 {
                self.steps = 1;
                res.arg = 4;
                res.addr_mode = [false, true];
                res.latch_stack = true;
                res.latch_ip = false;
            } else if self.steps == 1 {
                self.steps -= 1;
                res.arg = -4i16 as u16;
                res.addr_mode = [true, false];
                res.abs_jump = true;
            }
        } else if instr == 0x0C {
            res.latch_stack = true;
            res.alu_op = AluOperation::Right;
        } else if instr == 0x0D {
            res.alu_op = AluOperation::Right;
            res.latch_acc = true;
            res.io = res.arg == 0 && !res.addr_mode[1];
        } else if instr == 0x0E {
            if res.addr_mode[1] {
                panic!("addr_mode for save instruction must be direct or stack_rel");
            }
            res.write = true;
            res.io = res.arg == 4;
        } else if instr == 0x0F {
            if self.steps == 0 {
                self.steps = 1;
                res.alu_op = AluOperation::Right;
                res.latch_ip = false;
            } else if self.steps == 1 {
                self.steps -= 1;
                res.arg = temp_reg as u16;
                res.io = res.arg == 0;
                res.addr_mode = [false, false];
                res.alu_op = AluOperation::Right;
                res.latch_acc = true;
            }
        } else if instr == 0x10 {
            if self.steps == 0 {
                self.steps = 1;
                res.alu_op = AluOperation::Right;
                res.latch_ip = false;
            } else if self.steps == 1 {
                self.steps -= 1;
                res.arg = temp_reg as u16;
                res.io = res.arg == 4;
                res.addr_mode = [false, false];
                res.write = true;
            }
        } else if instr == 0x11 {
            res.halt = true;
        } else {
            panic!("Unexpected instruction");
        }

        res
    }
}

pub struct ControlUnit {
    pub datapath: DataPath,
    pub mem: [u32; 65536],
    pub ip: u16,
    pub temp_reg: u32,
    instruction_decoder: InstructionDecoder,
    pub instructions_count: usize,
    pub ticks_count: usize,
}

impl ControlUnit {
    pub fn new(data_mem: [u8; 65536], instr_mem: [u32; 65536]) -> Self {
        Self {
            datapath: DataPath::new(data_mem),
            mem: instr_mem,
            ip: 0,
            temp_reg: 0,
            instruction_decoder: InstructionDecoder { steps: 0 },
            instructions_count: 0,
            ticks_count: 0,
        }
    }

    pub fn tick(&mut self) -> bool {
        let word = self.mem[self.ip as usize];
        let res = self
            .instruction_decoder
            .process(word, self.ip, self.temp_reg);

        self.ticks_count += 1;
        if res.latch_ip {
            self.instructions_count += 1;
        }
        if res.halt {
            return true;
        }

        let (result, zero) = self.datapath.process(DataPathSignals {
            arg: res.arg,
            addr_mode: res.addr_mode,
            alu_op: res.alu_op,
            write: res.write,
            latch_acc: res.latch_acc,
            latch_stack: res.latch_stack,
            extend_arg: res.extend_arg,
            io: res.io,
        });

        self.temp_reg = result;

        let sum_ip_arg = self.ip.wrapping_add(word as u16);

        let mux_next_jmp = if (!res.if_zero | zero) & res.jmp {
            sum_ip_arg
        } else {
            self.ip.wrapping_add(1)
        };

        let mux_ip = if res.abs_jump {
            result as u16
        } else {
            mux_next_jmp
        };

        if res.latch_ip {
            self.ip = mux_ip;
        }

        false
    }
}

#[cfg(test)]
mod test {
    use std::collections::VecDeque;

    use super::ControlUnit;

    fn conf() -> ControlUnit {
        let cu = ControlUnit::new([0; 65536], [0; 65536]);
        cu
    }

    #[test]
    fn sign_spadd() {
        let mut cu = conf();
        cu.datapath.mem[(u16::MAX - 4) as usize] = 0;
        cu.datapath.mem[(u16::MAX - 8) as usize] = 1;
        cu.mem[0] = 0x0080F000; // sign 0xF000
        cu.mem[1] = 0x0E400000; // save ~ 0
        cu.mem[2] = 0x0C80FFFC; // spadd -4
        cu.mem[3] = 0x00807FFF; // sign 0x7FFF
        cu.mem[4] = 0x0E400000; // save ~ 0
        cu.mem[5] = 0x11000000; // halt
        while !cu.tick() {}
        assert_eq!(cu.datapath.mem[(u16::MAX - 4) as usize], 1);
        assert_eq!(cu.datapath.mem[(u16::MAX - 8) as usize], 0);
    }

    #[test]
    fn ops() {
        let mut v: u32 = 0;
        v += 5;
        v += 0x00038276u32;
        v += v;
        v &= 0x0F0F as u32;
        v |= 0xA0A0 as u32;
        v -= 0x00000276u32;
        v *= 3;
        v /= 2;
        v %= 11;
        let mut cu = conf();
        cu.datapath.mem[12] = 0x00038276u32.to_le_bytes()[0];
        cu.datapath.mem[13] = 0x00038276u32.to_le_bytes()[1];
        cu.datapath.mem[14] = 0x00038276u32.to_le_bytes()[2];
        cu.datapath.mem[15] = 0x00038276u32.to_le_bytes()[3];
        cu.datapath.mem[16] = 0x00000276u32.to_le_bytes()[0];
        cu.datapath.mem[17] = 0x00000276u32.to_le_bytes()[1];
        cu.datapath.mem[18] = 0x00000276u32.to_le_bytes()[2];
        cu.datapath.mem[19] = 0x00000276u32.to_le_bytes()[3];
        cu.mem[0] = 0x03800005; // add 0x0005
        cu.mem[1] = 0x0300000C; // add # 0x000C
        cu.mem[2] = 0x03F01234; // add acc
        cu.mem[3] = 0x01800F0F; // and 0x0F0F
        cu.mem[4] = 0x0280A0A0; // or 0xA0A0
        cu.mem[5] = 0x04000010; // sub # 0x0010
        cu.mem[6] = 0x05800003; // mul 0x0003
        cu.mem[7] = 0x06800002; // div 0x0002
        cu.mem[8] = 0x0780000B; // rem 0x000B
        cu.mem[9] = 0x11000000; // halt
        while !cu.tick() {}
        assert_eq!(cu.datapath.acc, v);
    }

    #[test]
    fn jumps() {
        let mut cu = conf();
        cu.mem[0] = 0x09000009; // jifz +9
        cu.mem[9] = 0x03800005; // add 5
        cu.mem[10] = 0x09000005; // jifz +5
        cu.mem[11] = 0x03800004; // add 4
        cu.mem[12] = 0x08000004; // jump +4
        cu.mem[15] = 0x11000000; // halt
        cu.mem[16] = 0x0800FFFF; // jump -1
        while !cu.tick() {}
        assert_eq!(cu.datapath.acc, 9);
    }

    #[test]
    fn call_ret() {
        let mut cu = conf();
        cu.mem[0] = 0x03800005; // add 5
        cu.mem[1] = 0x0E400000; // save ~ 0
        cu.mem[2] = 0x0A000020; // call 32
        cu.mem[3] = 0x0A000040; // call 64
        cu.mem[4] = 0x0D400000; // load ~ 0
        cu.mem[5] = 0x11000000; // halt

        cu.mem[32] = 0x0D400004; // load ~ 4
        cu.mem[33] = 0x03800006; // add 6
        cu.mem[34] = 0x0E400004; // save ~ 4
        cu.mem[35] = 0x0B000000; // ret

        cu.mem[64] = 0x0D400004; // load ~ 4
        cu.mem[65] = 0x03800007; // add 7
        cu.mem[66] = 0x0E400004; // save ~ 4
        cu.mem[67] = 0x0B000000; // ret

        while !cu.tick() {}
        assert_eq!(cu.datapath.acc, 18);
    }

    #[test]
    fn rel() {
        let mut cu = conf();
        cu.datapath.mem[12] = 16;
        cu.datapath.mem[16] = 0x12345678u32.to_le_bytes()[0];
        cu.datapath.mem[17] = 0x12345678u32.to_le_bytes()[1];
        cu.datapath.mem[18] = 0x12345678u32.to_le_bytes()[2];
        cu.datapath.mem[19] = 0x12345678u32.to_le_bytes()[3];
        cu.mem[0] = 0x0F00000C; // ldrel 12
        cu.mem[1] = 0x038000AA; // sum 0xAA
        cu.mem[2] = 0x1000000C; // svrel 12
        cu.mem[3] = 0x11000000; // halt
        while !cu.tick() {}
        assert_eq!(
            u32::from_le_bytes([
                cu.datapath.mem[16],
                cu.datapath.mem[17],
                cu.datapath.mem[18],
                cu.datapath.mem[19]
            ]),
            0x12345678u32 + 0xAAu32
        );
    }

    #[test]
    fn in_out() {
        let mut cu = conf();
        cu.datapath.input.append(&mut VecDeque::from_iter(
            "Hello".as_bytes().into_iter().map(|v| *v),
        ));
        cu.mem[0] = 0x0D000000; // load # 0
        cu.mem[1] = 0x0E000004; // save # 4
        cu.mem[2] = 0x0D000000; // load # 0
        cu.mem[3] = 0x0E000004; // save # 4
        cu.mem[4] = 0x0D000000; // load # 0
        cu.mem[5] = 0x0E000004; // save # 4
        cu.mem[6] = 0x0D000000; // load # 0
        cu.mem[7] = 0x0E000004; // save # 4
        cu.mem[8] = 0x0D000000; // load # 0
        cu.mem[9] = 0x0E000004; // save # 4
        cu.mem[10] = 0x11000000; // halt
        while !cu.tick() {}
        assert_eq!(std::str::from_utf8(&cu.datapath.output).unwrap(), "Hello");
    }
}
