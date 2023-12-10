use crate::{declare::Declared, expression::Expression, parser::parse};

fn declared_std() -> Declared {
    let mut declared = Declared::new();
    include_str!("../resources/built-in").lines().for_each(|v| {
        let mut words = v.split_ascii_whitespace();
        declared
            .fn_def(words.next().unwrap(), words.count())
            .unwrap();
    });

    declared
}

pub fn parse_std() -> (Vec<Expression>, Declared) {
    let mut declared = declared_std();
    let std_nl = include_str!("../resources/std.nl").to_string();
    let expressions = parse(std_nl, &mut declared).unwrap();

    (expressions, declared)
}
