mod parser;
mod declare;
mod std_expr;
mod expression;
mod util;

use std::{env, fs::File, error::Error, io::Read};

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

fn main() -> Result<(), Box<dyn Error>> {
    let (mut input, mut output) = parse_args()?;
    let mut input_str = String::new();
    input.read_to_string(&mut input_str)?;
    let (std_expressions, mut std_declared) = parse_std();
    let expressions = parser::parse(input_str, std_expressions, &mut std_declared)?;

    Ok(())
}