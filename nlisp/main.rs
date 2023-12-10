mod compiler;
mod declare;
mod expression;
mod parser;
mod preprocess;
mod std_expr;
mod util;

use std::{
    env,
    error::Error,
    fs::File,
    io::{Read, Write},
};

use compiler::compile;
use parser::parse;
use preprocess::{preprocess, Preprocessed};
use std_expr::parse_std;

fn parse_args() -> Result<(File, File), String> {
    let args: Vec<_> = env::args().collect();
    if args.len() < 3 {
        return Err("Not enough arguments".to_string());
    }

    let input = File::open(&args[1]).map_err(|_| "Can't open input file".to_string())?;
    let output = File::create(&args[2]).unwrap();

    Ok((input, output))
}

fn preprocessed_expressions(input_str: String) -> Result<Preprocessed, Box<dyn Error>> {
    let (mut std_expressions, mut std_declared) = parse_std();
    let expressions = parse(input_str, &mut std_declared)?;
    let mut preprocessed = preprocess(expressions);
    std_expressions.append(&mut preprocessed.fn_defs);
    preprocessed.fn_defs = std_expressions;

    Ok(preprocessed)
}

fn main() -> Result<(), Box<dyn Error>> {
    let (mut input, mut output) = parse_args()?;

    let mut input_str = String::new();
    input.read_to_string(&mut input_str)?;
    let code_lines_count = input_str.lines().count();

    let pe = preprocessed_expressions(input_str)?;
    let (bytecode, instructions_count, bytes_count) = compile(pe);

    output.write_all(&bytecode)?;
    eprintln!(
        "Code lines: {}; instructions: {}; bytes: {}",
        code_lines_count, instructions_count, bytes_count
    );

    Ok(())
}
