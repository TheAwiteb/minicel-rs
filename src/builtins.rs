use crate::ast::Expression;

pub type FunctionResult = Result<String, String>;

/// Call the builtin function. syntax: call_builtin![builtin_function1, builtin_function2, (name, args)]
/// Wich name is the name of the called function and args is the arguments of the function.
///
/// The builtin function must return a [`FunctionResult`].
macro_rules! call_builtin {
    ($($builtin_function: ident),+, ($call_name: ident, $call_args: ident)) => {
        match $call_name {
            $(
                stringify!($builtin_function) => Some($builtin_function($call_args)),
            )+
            _ => {
                log::error!("No builtin function found with name: {}", $call_name);
                None
            },
        }
    };
}

pub fn call_builtin(name: &str, args: Vec<Expression>) -> Option<FunctionResult> {
    log::debug!("Trying to call builtin function: {name} with args: {args:?}");
    call_builtin![print, sum, sub, mul, div, (name, args)]
}

pub fn is_builtin(name: &str) -> bool {
    ["print", "sum", "sub", "mul", "div"].contains(&name)
}

pub fn print(args: Vec<Expression>) -> FunctionResult {
    Ok(args
        .iter()
        .map(|arg| arg.to_string())
        .collect::<Vec<_>>()
        .join(", "))
}

pub fn sum(args: Vec<Expression>) -> FunctionResult {
    if args.len() != 2 {
        return Err(format!("Expected 2 arguments, found {}", args.len()));
    }
    match (&args[0], &args[1]) {
        (Expression::Number(n1), Expression::Number(n2)) => Ok((n1 + n2).to_string()),
        (a1, a2) => Err(format!("Expected numbers found `{a2}` and `{a1}`")),
    }
}

pub fn sub(args: Vec<Expression>) -> FunctionResult {
    if args.len() != 2 {
        return Err(format!("Expected 2 arguments, found {}", args.len()));
    }
    match (&args[0], &args[1]) {
        (Expression::Number(n1), Expression::Number(n2)) => Ok((n1 - n2).to_string()),
        (a1, a2) => Err(format!("Expected numbers found `{a2}` and `{a1}`")),
    }
}

pub fn mul(args: Vec<Expression>) -> FunctionResult {
    if args.len() != 2 {
        return Err(format!("Expected 2 arguments, found {}", args.len()));
    }
    match (&args[0], &args[1]) {
        (Expression::Number(n1), Expression::Number(n2)) => Ok((n1 * n2).to_string()),
        (a1, a2) => Err(format!("Expected numbers found `{a2}` and `{a1}`")),
    }
}

pub fn div(args: Vec<Expression>) -> FunctionResult {
    if args.len() != 2 {
        return Err(format!("Expected 2 arguments, found {}", args.len()));
    }
    match (&args[0], &args[1]) {
        (Expression::Number(n1), Expression::Number(n2)) => Ok((n1 / n2).to_string()),
        (a1, a2) => Err(format!("Expected numbers found `{a2}` and `{a1}`")),
    }
}
