use multipeek::MultiPeek;

use crate::ast::{Ast, Expression, FunctionCallExpression};
use crate::tokenizer::Token;

use crate::errors::{
    Error as MinicelError, ErrorKind as MinicelErrorKind, Result as MinicelResult,
};

/// The parser
#[derive(Debug)]
pub struct Parser<'a> {
    tokens: MultiPeek<std::slice::Iter<'a, Token>>,
    line_number: usize,
}

impl<'a> Parser<'a> {
    /// Creates a new parser from the given tokens.
    pub fn new(tokens: MultiPeek<std::slice::Iter<'a, Token>>, line_number: usize) -> Self {
        Self {
            tokens,
            line_number,
        }
    }

    /// Parses the tokens into an AST.
    pub fn parse(&mut self) -> MinicelResult<Ast> {
        log::debug!("Parsing tokens: {:#?}", self.tokens);

        let field_function = self.parse_function_call()?;
        let field_function = field_function
            .function_call()
            .expect("parse_function_call always returns a function call");
        Ok(Ast {
            function: field_function.clone(),
        })
    }

    /// Parses the identifier.
    fn parse_identifier(&mut self) -> MinicelResult<&str> {
        log::info!("Parsing identifier");

        match self.tokens.peek() {
            Some(Token::Identifier(identifier)) => {
                log::debug!("Found identifier: {identifier}");
                self.tokens.next();
                Ok(identifier)
            }
            Some(token) => {
                log::error!("Expected identifier token, found {:?}", token);
                Err(MinicelError::new(
                    MinicelErrorKind::Parse,
                    format!("Expected identifier, found {:?}", token),
                    self.line_number,
                ))
            }
            None => Err(MinicelError::new(
                MinicelErrorKind::Parse,
                "Expected identifier, found EOF".to_string(),
                self.line_number,
            )),
        }
    }

    /// Parses the field.
    /// A field is a identifier that represents a cell in the CSV file.
    /// e.g. `a1`, `fjkjfd34`, aa200` etc.
    fn parse_field(&mut self) -> MinicelResult<Expression> {
        log::info!("Parsing field");

        let identifier = self.parse_identifier()?;
        let col = identifier
            .chars()
            .take_while(|c| c.is_ascii_alphabetic())
            .collect::<String>();
        log::debug!("Found column in the field: {col}");
        let row = identifier
            .chars()
            .skip_while(|c| c.is_ascii_alphabetic())
            .collect::<String>();
        log::debug!("Found row in the field: {row}");
        let row = match row.parse() {
            Ok(row) => {
                if row == 0 {
                    return Err(MinicelError::new(
                        MinicelErrorKind::Parse,
                        "Invalid field identifier, row number starts from 1, found 0".to_owned(),
                        self.line_number,
                    ));
                }
                row
            },
            Err(_) => {
                return Err(MinicelError::new(
                    MinicelErrorKind::Parse,
                    format!(
                        "Invalid field identifier, expected a row number after the column `{col}` but found `{row}`"
                    ),
                    self.line_number,
                ))
            }
        };

        Ok(Expression::Field {
            col,
            row,
            value: String::new(),
        })
    }

    /// Parses the array.
    fn parse_array(&mut self) -> MinicelResult<Expression> {
        log::info!("Parsing array");

        let mut array = Vec::new();
        match self.tokens.peek() {
            Some(Token::LeftBracket) => {
                log::info!("Found left bracket");
                self.tokens.next();
                while let Some(token) = self.tokens.peek() {
                    match token {
                        Token::RightBracket => {
                            log::info!("Found right bracket");
                            self.tokens.next();
                            return Ok(Expression::Array(array));
                        }
                        Token::Semicolon => {
                            self.tokens.next();
                        }
                        _ => {
                            log::info!("Parsing expression in array");
                            array.push(self.parse_expression()?);
                        }
                    }
                }
                log::error!("Expected right bracket, found EOF");
                Err(MinicelError::new(
                    MinicelErrorKind::Parse,
                    "Expected right bracket, found EOF".to_string(),
                    self.line_number,
                ))
            }
            Some(token) => Err(MinicelError::new(
                MinicelErrorKind::Parse,
                format!("Expected left bracket, found {token:?}"),
                self.line_number,
            )),
            None => Err(MinicelError::new(
                MinicelErrorKind::Parse,
                "Expected left bracket, found EOF".to_string(),
                self.line_number,
            )),
        }
    }

    /// Parses the expression.
    fn parse_expression(&mut self) -> MinicelResult<Expression> {
        log::info!("Parsing expression");

        match self.tokens.peek() {
            Some(token) => {
                log::debug!("Found token: {token:?}");
                match token {
                    Token::Identifier(ident) => {
                        if self.tokens.peek_nth(1) == Some(&&Token::LeftParenthesis) {
                            log::info!("Found indentifer followed by left parenthesis, parsing function call");
                            self.parse_function_call()
                        } else if ident == "true" || ident == "false" {
                            log::info!("Found identifier that is a boolean: {ident}");
                            self.tokens.next();
                            Ok(Expression::Boolean(ident == "true"))
                        } else {
                            log::info!("Found identifier that is not a function call and not a boolean, parsing field");
                            self.parse_field()
                        }
                    }
                    Token::Number(n) => {
                        log::debug!("Found number: {n}");
                        self.tokens.next();
                        Ok(Expression::Number(n.clone()))
                    }
                    Token::String(s) => {
                        log::debug!("Found string: {s}");
                        self.tokens.next();
                        Ok(Expression::String(s.clone()))
                    }
                    Token::LeftBracket => {
                        log::info!("Found left bracket, parsing array");
                        self.parse_array()
                    }
                    _ => Err(MinicelError::new(
                        MinicelErrorKind::Parse,
                        format!("Expected expression, found {:?}", token),
                        self.line_number,
                    )),
                }
            }
            None => Err(MinicelError::new(
                MinicelErrorKind::Parse,
                "Expected expression, found EOF".to_string(),
                self.line_number,
            )),
        }
    }

    /// Parses the arguments.
    fn parse_arguments(&mut self) -> MinicelResult<Vec<Expression>> {
        log::info!("Parsing function arguments");

        let mut arguments = Vec::new();
        match self.tokens.peek() {
            Some(Token::LeftParenthesis) => {
                log::info!("Found left parenthesis");
                self.tokens.next();
                while let Some(token) = self.tokens.peek() {
                    match token {
                        Token::RightParenthesis => {
                            log::info!("Found right parenthesis, returning arguments");
                            self.tokens.next();
                            return Ok(arguments);
                        }
                        Token::Semicolon => {
                            log::info!("Found semicolon");
                            self.tokens.next();
                        }
                        c => {
                            log::debug!("Found token: {c:?} and parsing it as an expression");
                            arguments.push(self.parse_expression()?);
                        }
                    }
                }
                Err(MinicelError::new(
                    MinicelErrorKind::Parse,
                    "Expected right parenthesis, found EOF".to_string(),
                    self.line_number,
                ))
            }
            Some(token) => Err(MinicelError::new(
                MinicelErrorKind::Parse,
                format!("Expected left parenthesis, found {:?}", token),
                self.line_number,
            )),
            None => Err(MinicelError::new(
                MinicelErrorKind::Parse,
                "Expected left parenthesis, found EOF".to_string(),
                self.line_number,
            )),
        }
    }

    /// Parses the function call.
    fn parse_function_call(&mut self) -> MinicelResult<Expression> {
        log::info!("Parsing function call");

        let name = self.parse_identifier()?.to_string();
        let arguments = self.parse_arguments()?;
        Ok(Expression::FunctionCall(FunctionCallExpression {
            name: name.to_string(),
            arguments,
            line_number: self.line_number,
        }))
    }
}
