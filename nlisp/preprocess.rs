use crate::expression::Expression;

#[derive(Debug)]
pub struct Preprocessed {
    pub fn_defs: Vec<Expression>,
    pub main: Vec<Expression>,
}

fn preprocess_expr(
    expression: Expression,
    preprocessed: &mut Preprocessed,
) -> Expression {
    match expression {
        Expression::FnDef {
            name,
            arguments,
            expr,
        } => {
            let expr = Box::new(preprocess_expr(*expr, preprocessed));
            preprocessed
                .fn_defs
                .push(Expression::FnDef { name, arguments, expr });
            Expression::Value(0)
        }
        Expression::Case { condition, t, f } => {
            let condition = Box::new(preprocess_expr(*condition, preprocessed));
            let t = Box::new(preprocess_expr(*t, preprocessed));
            let f = Box::new(preprocess_expr(*f, preprocessed));
            Expression::Case { condition, t, f }
        }
        Expression::For {
            var,
            next_val,
            while_expr,
            expr,
        } => {
            let next_val = Box::new(preprocess_expr(*next_val, preprocessed));
            let while_expr = Box::new(preprocess_expr(*while_expr, preprocessed));
            let expr = Box::new(preprocess_expr(*expr, preprocessed));
            Expression::For { var, next_val, while_expr, expr }
        },
        Expression::Fn { name, args } => {
            let args: Vec<Expression> = args.into_iter().map(|v| preprocess_expr(v, preprocessed)).collect();
            Expression::Fn { name, args }
        },
        Expression::VarDef { name, init, expr } => {
            let init = Box::new(preprocess_expr(*init, preprocessed));
            let expr = Box::new(preprocess_expr(*expr, preprocessed));
            Expression::VarDef { name, init, expr }
        }
        _ => {
            expression
        }
    }
}

pub fn preprocess(program: Vec<Expression>) -> Preprocessed {
    let mut preprocessed = Preprocessed {
        fn_defs: Vec::new(),
        main: Vec::new(),
    };
    let program: Vec<Expression> = program
        .into_iter()
        .map(|v| preprocess_expr(v, &mut preprocessed))
        .collect();
    preprocessed.main = program;
    preprocessed
}