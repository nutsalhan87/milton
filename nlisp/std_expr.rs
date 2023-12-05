use std::fs;

use crate::{declare::Declared, expression::Expression, parser::parse};

fn declared_std() -> Declared {
    let mut declared = Declared::new();
    fs::read_to_string("resources/built-in").unwrap().lines().for_each(|v| {
        let mut words = v.split_ascii_whitespace();
        declared.fn_def(words.next().unwrap(), words.count()).unwrap();
    });
    declared
}

pub fn parse_std() -> (Vec<Expression>, Declared) {
    let mut declared = declared_std();
    let std_nl = fs::read_to_string("resources/std.nl").unwrap();
    let expressions = parse(std_nl, &mut declared).unwrap();

    (expressions, declared)
}