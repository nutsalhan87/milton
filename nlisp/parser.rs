use crate::{
    expression::{Expression, ExpressionType},
    util::{error_code, replace_n, split_first},
};

use super::declare::Declared;

fn parse_char(expr_pointer: &mut &str) -> Result<Expression, String> {
    *expr_pointer = expr_pointer.trim_start();
    if expr_pointer.as_bytes()[0] != b'\'' || expr_pointer.as_bytes()[2] != b'\'' {
        return Err(ExpressionType::Char.to_explained_string(expr_pointer));
    }

    let chr = expr_pointer.as_bytes()[1];
    *expr_pointer = &expr_pointer[3..];

    Ok(Expression::Value(chr.into()))
}

fn parse_num(expr_pointer: &mut &str) -> Result<Expression, String> {
    *expr_pointer = expr_pointer.trim_start();
    let (num, other) = split_first(expr_pointer, &[' ', ')'], ExpressionType::Number)?;
    let num = num
        .parse::<i32>()
        .map_err(|_| ExpressionType::Number.to_explained_string(expr_pointer))?;
    *expr_pointer = other;

    Ok(Expression::Value(num))
}

fn parse_str(expr_pointer: &mut &str) -> Result<Expression, String> {
    *expr_pointer = expr_pointer.trim_start();
    if expr_pointer.as_bytes()[0] != b'\"' {
        return Err(ExpressionType::Literal.to_explained_string(expr_pointer));
    }

    let (literal, other) = split_first(&expr_pointer[1..], &['\"'], ExpressionType::Literal)?;
    *expr_pointer = &other[1..];

    Ok(Expression::Str(literal.to_string()))
}

fn parse_arg_names(
    expr_pointer: &mut &str,
    declared: &mut Declared,
) -> Result<Vec<String>, String> {
    *expr_pointer = expr_pointer.trim_start();
    if expr_pointer.as_bytes()[0] != b'(' {
        return Err(ExpressionType::Args.to_explained_string(expr_pointer));
    }

    let (args, other) = split_first(&expr_pointer[1..], &[')'], ExpressionType::Args)?;
    *expr_pointer = &other[1..];

    let mut args_splitted = Vec::new();
    for arg in args.split_ascii_whitespace() {
        declared.var_dec(arg)?;
        args_splitted.push(arg.to_string());
    }

    Ok(args_splitted)
}

fn parse_fn_def(expr_pointer: &mut &str, declared: &mut Declared) -> Result<Expression, String> {
    let (fn_name, other) = split_first(
        expr_pointer.trim_start()[2..].trim_start(),
        &[' '],
        ExpressionType::FnDef,
    )?;
    *expr_pointer = other;

    let mut novar = declared.novar();
    let arg_names = parse_arg_names(expr_pointer, &mut novar)?;
    novar.fn_def(fn_name, arg_names.len())?;

    let expr = Box::new(parse_expr(expr_pointer, &mut novar)?);
    declared.fns = novar.fns;

    Ok(Expression::FnDef {
        name: fn_name.to_string(),
        arguments: arg_names,
        expr,
    })
}

fn parse_var_def(expr_pointer: &mut &str, declared: &mut Declared) -> Result<Expression, String> {
    let (var_name, other) = split_first(
        expr_pointer.trim_start()[3..].trim_start(),
        &[' '],
        ExpressionType::VarDef,
    )?;
    *expr_pointer = other;

    let init = Box::new(parse_expr(expr_pointer, declared)?);
    declared.var_dec(var_name)?;

    let expr = Box::new(parse_expr(expr_pointer, declared)?);
    declared.var_undec(var_name)?;

    Ok(Expression::VarDef {
        name: var_name.to_string(),
        init,
        expr,
    })
}

fn parse_case(expr_pointer: &mut &str, declared: &mut Declared) -> Result<Expression, String> {
    *expr_pointer = &expr_pointer.trim_start()[4..];

    let condition = Box::new(parse_expr(expr_pointer, declared)?);
    let t = Box::new(parse_expr(expr_pointer, declared)?);
    let f = Box::new(parse_expr(expr_pointer, declared)?);

    Ok(Expression::Case { condition, t, f })
}

fn parse_for(expr_pointer: &mut &str, declared: &mut Declared) -> Result<Expression, String> {
    let (var, other) = split_first(
        expr_pointer.trim_start()[3..].trim_start(),
        &[' '],
        ExpressionType::For,
    )?;
    *expr_pointer = other;

    declared.var_dec(var)?;
    let next_val = Box::new(parse_expr(expr_pointer, declared)?);
    let while_expr = Box::new(parse_expr(expr_pointer, declared)?);
    let expr = Box::new(parse_expr(expr_pointer, declared)?);
    declared.var_undec(var)?;

    Ok(Expression::For {
        var: var.to_string(),
        next_val,
        while_expr,
        expr,
    })
}

fn parse_fn_or_var(expr_pointer: &mut &str, declared: &mut Declared) -> Result<Expression, String> {
    let (name, other) = split_first(
        expr_pointer.trim_start(),
        &[' ', ')'],
        ExpressionType::FnOrVar,
    )?;
    *expr_pointer = other;

    if let Some(args_count) = declared.fns.get(name) {
        let mut args: Vec<Expression> = Vec::new();
        for _ in 0..*args_count {
            args.push(parse_expr(expr_pointer, declared)?);
        }

        Ok(Expression::Fn {
            name: name.to_string(),
            args,
        })
    } else if declared.vars.contains(name) {
        Ok(Expression::Var(name.to_string()))
    } else {
        Err(format!(
            "Variable '{}' did not declared: {}...",
            &name,
            error_code(expr_pointer)
        ))
    }
}

fn parse_expr(expr_pointer: &mut &str, declared: &mut Declared) -> Result<Expression, String> {
    *expr_pointer = expr_pointer.trim_start();
    match expr_pointer
        .chars()
        .next()
        .ok_or_else(|| format!("Parsing error: {}...", error_code(expr_pointer)))?
    {
        '(' => {
            *expr_pointer = &expr_pointer[1..];
            let expr = parse_expr(expr_pointer, declared);
            *expr_pointer = &expr_pointer.trim_start()[1..];
            return expr;
        }
        '\'' => {
            return parse_char(expr_pointer);
        }
        '\"' => {
            return parse_str(expr_pointer);
        }
        _ => (),
    }

    match expr_pointer
        .split([' ', ')'])
        .next()
        .ok_or_else(|| format!("Parsing error: {}...", error_code(expr_pointer)))?
    {
        "fn" => parse_fn_def(expr_pointer, declared),
        "case" => parse_case(expr_pointer, declared),
        "for" => parse_for(expr_pointer, declared),
        "let" => parse_var_def(expr_pointer, declared),
        other => {
            if other.parse::<i32>().is_ok() {
                parse_num(expr_pointer)
            } else {
                parse_fn_or_var(expr_pointer, declared)
            }
        }
    }
}

pub fn parse(input: String, std_declared: &mut Declared) -> Result<Vec<Expression>, String> {
    let input = replace_n(input);
    let input_pointer: &mut &str = &mut &input[..];
    let mut expressions = Vec::new();
    while !input_pointer.trim().is_empty() && input_pointer.trim_start().as_bytes()[0] == b'(' {
        expressions.push(parse_expr(input_pointer, std_declared)?);
    }

    Ok(expressions)
}
