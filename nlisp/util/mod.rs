pub mod declared;
pub mod expression;
pub mod std_expr;

use std::cmp::min;

use self::expression::ExpressionType;

pub fn replace_n(s: String) -> String {
    s.split('\"')
        .enumerate()
        .map(|(idx, sub)| {
            if idx % 2 == 0 {
                sub.replace('\n', " ")
            } else {
                sub.to_string()
            }
        })
        .fold("".to_string(), |acc, v| format!("{acc}\"{v}"))[1..]
        .to_string()
}

pub fn error_code(s: &str) -> &str {
    &s[..min(30, s.len())]
}

pub fn split_first<'a>(
    s: &'a str,
    p: &[char],
    expr_type: ExpressionType,
) -> Result<(&'a str, &'a str), String> {
    let len = s.find(p).ok_or_else(|| expr_type.to_explained_string(s))?;
    Ok((&s[..len], &s[len..]))
}
