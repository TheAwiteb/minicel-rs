use bigdecimal::BigDecimal;

/// The expressions.
#[derive(Debug, Clone)]
pub enum Expression {
    /// A function call. e.g. `add(a1, add(a2, a3))`
    FunctionCall(FunctionCallExpression),
    /// A field. e.g. `a1`
    Field {
        col: String,
        row: u64,
        value: String,
    },
    /// A number. e.g. `1` or `-2` or `1.10`
    Number(BigDecimal),
    /// A string. e.g. `"hello"`
    String(String),
    /// A boolean. `true` or `false`
    Boolean(bool),
    /// A array. e.g. `[1, 2, 3, add(a1, a2)]`
    Array(Vec<Expression>),
}

/// The function call expression.
#[derive(Debug, Clone)]
pub struct FunctionCallExpression {
    pub name: String,
    pub arguments: Vec<Expression>,
    pub line_number: usize,
}

/// The AST of the field.
#[derive(Debug, Clone)]
pub struct Ast {
    pub function: FunctionCallExpression,
}

impl Ast {
    /// Returns the children of the AST.
    pub fn mut_children(&mut self) -> Vec<&mut Expression> {
        let mut children = Vec::new();
        for argument in &mut self.function.arguments {
            children.extend(argument.mut_children());
        }
        children
    }
}

impl Expression {
    /// Returns the function call expression. if expression is not a function call, returns None.
    pub fn function_call(&self) -> Option<&FunctionCallExpression> {
        match self {
            Expression::FunctionCall(function_call) => Some(function_call),
            _ => None,
        }
    }

    /// Returns the children of the expression.
    pub fn mut_children(&mut self) -> Vec<&mut Expression> {
        let mut children = Vec::new();
        match self {
            Expression::FunctionCall(function_call) => {
                for argument in &mut function_call.arguments {
                    children.extend(argument.mut_children());
                }
            }
            Expression::Array(array) => {
                for element in array {
                    children.extend(element.mut_children());
                }
            }
            c => {
                children.push(c);
            }
        }
        children
    }
}

impl std::fmt::Display for Expression {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Expression::FunctionCall(function_call) => {
                write!(
                    f,
                    "{}",
                    if crate::builtins::is_builtin(&function_call.name) {
                        format!("builtin function: {}", function_call.name)
                    } else {
                        format!("function: {}", function_call.name)
                    }
                )?;
                write!(f, "(")?;
                for (i, argument) in function_call.arguments.iter().enumerate() {
                    write!(f, "{}", argument)?;
                    if i != function_call.arguments.len() - 1 {
                        write!(f, ", ")?;
                    }
                }
                write!(f, ")")
            }
            Expression::Field {
                col: _,
                row: _,
                value,
            } => write!(f, "{value}"),
            Expression::Number(number) => write!(f, "{}", number),
            Expression::String(string) => write!(f, "{}", string),
            Expression::Boolean(boolean) => write!(f, "{}", boolean),
            Expression::Array(array) => {
                write!(f, "[")?;
                for (i, element) in array.iter().enumerate() {
                    write!(f, "{}", element)?;
                    if i != array.len() - 1 {
                        write!(f, ", ")?;
                    }
                }
                write!(f, "]")
            }
        }
    }
}
