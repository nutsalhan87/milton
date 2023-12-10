pub fn decode_asm(instr: u32) -> String {
    match (instr >> 24) as u8 {
        0x00 => format!("sign {}", addr_mode_str(instr)),
        0x01 => format!("and {}", addr_mode_str(instr)),
        0x02 => format!("or {}", addr_mode_str(instr)),
        0x03 => format!("add {}", addr_mode_str(instr)),
        0x04 => format!("sub {}", addr_mode_str(instr)),
        0x05 => format!("mul {}", addr_mode_str(instr)),
        0x06 => format!("div {}", addr_mode_str(instr)),
        0x07 => format!("rem {}", addr_mode_str(instr)),
        0x08 => format!("jump {}", instr as u16 as i16),
        0x09 => format!("jifz {}", instr as u16 as i16),
        0x0A => format!("call {}", instr as u16 as i16),
        0x0B => "ret".to_string(),
        0x0C => format!("spadd {}", addr_mode_str(instr)),
        0x0D => format!("load {}", addr_mode_str(instr)),
        0x0E => format!("save {}", addr_mode_str(instr)),
        0x0F => format!("ldrel {}", addr_mode_str(instr)),
        0x10 => format!("svrel {}", addr_mode_str(instr)),
        0x11 => "halt".to_string(),
        _ => panic!("Invalid instruction: {}", instr),
    }
}

fn addr_mode_str(instr: u32) -> String {
    let arg = instr as u16;
    let addr_mode = (instr >> 22) & 0x3;
    match addr_mode {
        0b00 => format!("# {}", arg as i16),
        0b01 => format!("~ {}", arg as i16),
        0b10 => format!("{}", arg as i16),
        0b11 => "acc".to_string(),
        _ => panic!("Invalid addresation mode"),
    }
}
