use std::collections::HashMap;

use crate::{expression::Expression, preprocess::Preprocessed};

fn built_in() -> (Vec<u32>, HashMap<String, u16>) {
    let mut lines = include_str!("../resources/built-in-asm").lines();
    let mut instructions: Vec<u32> = vec![0];
    let mut fn_addresses: HashMap<String, u16> = HashMap::new();

    while let Some(line) = lines.next() {
        let mut words = line.split_ascii_whitespace();
        let fn_name = words.next().unwrap().to_string();
        fn_addresses.insert(fn_name, instructions.len() as u16);
        for _ in 0..words.next().unwrap().parse::<usize>().unwrap() {
            instructions.push(
                u32::from_str_radix(lines.next().unwrap().split_whitespace().next().unwrap(), 16)
                    .unwrap(),
            );
        }
    }

    (instructions, fn_addresses)
}

enum Var {
    Memory(u16),
    Stack(u16),
    InWord(u16),
}

impl Var {
    fn to_arg(&self) -> u32 {
        match self {
            Var::Memory(arg) => *arg as u32,
            Var::Stack(arg) => *arg as u32 | 0x00400000u32,
            Var::InWord(arg) => *arg as u32 | 0x00800000u32,
        }
    }
}

fn translate(
    expr: &Expression,
    vars: &mut HashMap<String, Var>,
    fn_addresses: &HashMap<String, u16>,
    data: &mut Vec<u8>,
) -> Vec<u32> {
    match expr {
        Expression::FnDef {
            name: _,
            arguments,
            expr,
        } => {
            let mut fn_vars: HashMap<String, Var> = HashMap::from_iter(
                arguments
                    .iter()
                    .rev()
                    .enumerate()
                    .map(|(i, v)| (v.clone(), Var::Stack((i + 1) as u16 * 4))),
            );
            let mut instructions = translate(expr, &mut fn_vars, fn_addresses, data);
            instructions.push(0x0B000000); // ret

            instructions
        }
        Expression::Case { condition, t, f } => {
            let mut c_instructions = translate(condition, vars, fn_addresses, data);
            let mut t_instructions = translate(t, vars, fn_addresses, data);
            let mut f_instructions = translate(f, vars, fn_addresses, data);
            t_instructions.push(0x08000000 + f_instructions.len() as u32 + 1); // jump -> end
            c_instructions.push(0x09000000 + t_instructions.len() as u32 + 1); // jifz -> end
            c_instructions.append(&mut t_instructions);
            c_instructions.append(&mut f_instructions);

            c_instructions
        }
        Expression::For {
            var,
            next_val,
            while_expr,
            expr,
        } => {
            let mut instructions = Vec::new();
            vars.insert(var.clone(), Var::Memory(data.len() as u16 + 12)); // variable
            data.append(&mut vec![0, 0, 0, 0]);

            instructions.push(0x0D800000); // load 0
            instructions.push(0x0E000000 | vars[var].to_arg()); // save # var
            instructions.push(0x0C80FFFC); // spadd -4
            instructions.push(0x0E400000); // save ~ 0 - init cumulative with 0

            vars.iter_mut().for_each(|v| {
                if let Var::Stack(n) = v.1 {
                    *n += 4;
                }
            });

            let next_val_addr = instructions.len();

            let mut next_val_instructions = translate(next_val, vars, fn_addresses, data);
            instructions.append(&mut next_val_instructions);
            instructions.push(0x0E000000 | vars[var].to_arg()); // save # var

            let mut while_instructions = translate(while_expr, vars, fn_addresses, data);
            instructions.append(&mut while_instructions);

            let mut expr_instructions = translate(expr, vars, fn_addresses, data);
            instructions.push(0x09000000 + expr_instructions.len() as u32 + 4); // jifz + -> end
            instructions.append(&mut expr_instructions);
            instructions.push(0x03400000); // add ~ 0
            instructions.push(0x0E400000); // save ~ 0
            instructions.push(
                0x08000000
                    | (next_val_addr as i32 - instructions.len() as i32) as i16 as u16 as u32,
            ); // jump - -> next_val
            instructions.push(0x0D400000); // load ~ 0 :end
            instructions.push(0x0C800004); // spadd 4

            vars.iter_mut().for_each(|v| {
                if let Var::Stack(n) = v.1 {
                    *n -= 4;
                }
            });
            vars.remove(var);

            instructions
        }
        Expression::Fn { name, args } => {
            let mut instructions = Vec::new();
            instructions.push(0x0C800000 | (args.len() as i16 * -4) as u16 as u32); // spadd -x, where x = args.len() * 4

            vars.iter_mut().for_each(|v| {
                if let Var::Stack(n) = v.1 {
                    *n += args.len() as u16 * 4;
                }
            });

            for (idx, arg) in args
                .iter()
                .enumerate()
                .map(|(idx, v)| (args.len() - idx - 1, v))
            {
                let mut arg_instructions = translate(arg, vars, fn_addresses, data);
                instructions.append(&mut arg_instructions);
                instructions.push(0x0E400000 + 4 * idx as u32); // save ~n
            }
            instructions.push(0x0A000000 | fn_addresses[name] as u32); // call -
            instructions.push(0x0C800000 | (args.len() * 4) as u32); // spadd x

            vars.iter_mut().for_each(|v| {
                if let Var::Stack(n) = v.1 {
                    *n -= args.len() as u16 * 4;
                }
            });

            instructions
        }
        Expression::VarDef { name, init, expr } => {
            let mut instructions = Vec::new();

            vars.insert(name.clone(), Var::Memory(data.len() as u16 + 12)); // variable
            data.append(&mut vec![0, 0, 0, 0]);

            let mut init_instructions = translate(init, vars, fn_addresses, data);
            instructions.append(&mut init_instructions);
            instructions.push(0x0E000000 | vars[name].to_arg()); // save # x, where x is var addr

            let mut expr_instructions = translate(expr, vars, fn_addresses, data);
            instructions.append(&mut expr_instructions);

            vars.remove(name);

            instructions
        }
        Expression::Var(name) => {
            vec![0x0D000000 | vars[name].to_arg()] // load
        }
        Expression::Str(s) => {
            let pointer = data.len() + 12;
            data.append(&mut Vec::from(s.clone().as_bytes()));
            data.push(0);

            vec![0x0D800000 | pointer as u32] // load
        }
        Expression::Value(num) => {
            if let Ok(num) = i16::try_from(*num) {
                vec![0x0D000000 | Var::InWord(num as u16).to_arg()] // load
            } else {
                let var = Var::Memory(data.len() as u16 + 12);
                data.append(&mut Vec::from(num.to_le_bytes()));

                vec![0x0D000000 | var.to_arg()] // load
            }
        }
    }
}

pub fn compile(preprocessed: Preprocessed) -> (Vec<u8>, usize, usize) {
    let (mut instructions, mut fn_addresses) = built_in();
    let mut data: Vec<u8> = Vec::new();
    let mut vars = HashMap::new();

    for fn_def in preprocessed.fn_defs {
        let mut fn_def_asm = translate(&fn_def, &mut vars, &fn_addresses, &mut data);
        if let Expression::FnDef { name, .. } = fn_def {
            fn_addresses.insert(name, instructions.len() as u16);
        } else {
            panic!("There must be function definition");
        }
        instructions.append(&mut fn_def_asm);
    }
    instructions[0] = 0x08000000 + instructions.len() as u32; // jump n

    for expr in preprocessed.main {
        let mut expr_instructions = translate(&expr, &mut vars, &fn_addresses, &mut data);
        instructions.append(&mut expr_instructions);
    }
    instructions.push(0x11000000); // halt

    let instructions_count = instructions.len();
    for (idx, instr) in instructions.iter().copied().enumerate() {
        eprintln!("{idx} {instr:08x} {}", vm::decode_asm(instr));
    }

    let mut data = [vec![0u8; 12], data].concat();
    data[8] = (data.len() as u32).to_le_bytes()[0];
    data[9] = (data.len() as u32).to_le_bytes()[1];
    data[10] = (data.len() as u32).to_le_bytes()[2];
    data[11] = (data.len() as u32).to_le_bytes()[3];

    let bytecode = [
        data,
        instructions
            .into_iter()
            .flat_map(|v| v.to_le_bytes())
            .collect(),
    ]
    .concat();
    let bytes_count = bytecode.len();

    (bytecode, instructions_count, bytes_count)
}
