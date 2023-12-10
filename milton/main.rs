use std::{collections::VecDeque, env, error::Error, fs::File, io::Read};

use controlunit::ControlUnit;

mod controlunit;
mod datapath;

struct Args {
    file: File,
    input: String,
}

fn parse_args() -> Result<Args, Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        Err("Not enough arguments")?
    }

    let file = File::open(&args[1])?;
    let input = args[2..].join(" ");

    Ok(Args { file, input })
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = parse_args()?;

    let mut bytecode = Vec::new();
    args.file.read_to_end(&mut bytecode)?;

    let data_size = u32::from_le_bytes(bytecode[8..12].try_into()?) as usize;

    let mut data_mem = bytecode[..data_size].to_vec();
    data_mem.resize(65536, 0);

    let mut instructions_mem: Vec<u32> = bytecode[data_size..]
        .chunks(4)
        .map(|v| u32::from_le_bytes(v.try_into().unwrap()))
        .collect();
    instructions_mem.resize(65536, 0);

    let mut cu = ControlUnit::new(
        data_mem.try_into().unwrap(),
        instructions_mem.try_into().unwrap(),
    );
    cu.datapath.input = VecDeque::from_iter(args.input.as_bytes().iter().copied());

    while !cu.tick() {
        eprintln!(
            "{}        ip: {}, acc: {}, sp: {}",
            vm::decode_asm(cu.mem[cu.ip as usize], None),
            cu.ip,
            cu.datapath.acc,
            cu.datapath.sp
        );
    }
    eprintln!(
        "Ticks: {}; instructions: {}",
        cu.ticks_count, cu.instructions_count
    );

    println!("{}", std::str::from_utf8(&cu.datapath.output)?);

    Ok(())
}
