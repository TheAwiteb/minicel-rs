//! The tokenizer of the minicel-rs, this tokenizer only tokenizes the functions calls and its arguments.

use std::{
    iter::Peekable,
    str::{Chars, FromStr},
};

use bigdecimal::BigDecimal;

use crate::errors::{
    Error as MinicelError, ErrorKind as MinicelErrorKind, Result as MinicelResult,
};

/// The tokens that the tokenizer can produce.
#[derive(Debug, PartialEq)]
pub enum Token {
    /// A Identifier token, this token is used to represent the function name and the record and boolean arguments of the function.
    Identifier(String),
    /// String token, this token is used to represent the string arguments of the function.
    String(String),
    /// Number token, this token is used to represent the number arguments of the function.
    Number(BigDecimal),
    /// Semicolon token, this token is used to represent the semicolon that separates the arguments of the function.
    Semicolon,
    /// Left Parenthesis token, this token is used to represent the left parenthesis that opens the function call.
    LeftParenthesis,
    /// Right Parenthesis token, this token is used to represent the right parenthesis that closes the function call.
    RightParenthesis,
    /// Left Bracket token, this token is used to represent the open of array.
    LeftBracket,
    /// Right Bracket token, this token is used to represent the close of array.
    RightBracket,
}

/// Read the string
fn read_string(field: &mut Peekable<Chars<'_>>, line_number: usize) -> MinicelResult<Token> {
    let mut string = String::new();
    let mut is_closed = false;
    for c in field.by_ref() {
        if c == '"' {
            is_closed = true;
            break;
        }
        string.push(c);
    }
    if is_closed {
        Ok(Token::String(string))
    } else {
        Err(MinicelError::new(
            MinicelErrorKind::Tokenizer,
            "String is not closed".to_string(),
            line_number,
        ))
    }
}

/// Read the number
fn read_number(field: &mut Peekable<Chars<'_>>, line_number: usize) -> MinicelResult<Token> {
    let mut number = String::new();
    let mut is_float = false;
    let mut is_negative = false;
    while let Some(c) = field.peek() {
        match c {
            '0'..='9' => {
                number.push(*c);
                field.next();
            }
            '.' => {
                if is_float {
                    return Err(MinicelError::new(
                        MinicelErrorKind::Tokenizer,
                        "Invalid float number".to_string(),
                        line_number,
                    ));
                }
                is_float = true;
                number.push(*c);
                field.next();
            }
            '-' => {
                if is_negative || !number.is_empty() {
                    return Err(MinicelError::new(
                        MinicelErrorKind::Tokenizer,
                        "Invalid negative number".to_string(),
                        line_number,
                    ));
                }
                is_negative = true;
                number.push(*c);
                field.next();
            }
            _ => break,
        }
    }

    Ok(Token::Number(
        BigDecimal::from_str(&number).expect("is a number"),
    ))
}

/// Read the identifier
fn read_identifier(field: &mut Peekable<Chars<'_>>) -> Token {
    let mut identifier = String::new();
    while let Some(c) = field.peek() {
        match c {
            '_' | 'a'..='z' | 'A'..='Z' | '0'..='9' => {
                identifier.push(*c);
                field.next();
            }
            _ => break,
        }
    }
    Token::Identifier(identifier)
}

/// Tokenize the given field.
pub fn tokenize(field: &str, line_number: usize) -> MinicelResult<Vec<Token>> {
    let mut field = field.chars().peekable();
    let mut tokens = Vec::new();
    while let Some(c) = field.peek() {
        match c {
            ';' => {
                tokens.push(Token::Semicolon);
                field.next();
            }
            '(' => {
                tokens.push(Token::LeftParenthesis);
                field.next();
            }
            ')' => {
                tokens.push(Token::RightParenthesis);
                field.next();
            }
            '[' => {
                tokens.push(Token::LeftBracket);
                field.next();
            }
            ']' => {
                tokens.push(Token::RightBracket);
                field.next();
            }
            '"' => {
                field.next();
                tokens.push(read_string(&mut field, line_number)?);
            }
            '0'..='9' | '-' => {
                tokens.push(read_number(&mut field, line_number)?);
            }
            '_' | 'a'..='z' | 'A'..='Z' => {
                tokens.push(read_identifier(&mut field));
            }
            c if c.is_whitespace() => {
                field.next();
            }
            _ => {
                return Err(MinicelError::new(
                    MinicelErrorKind::Tokenizer,
                    format!("Unknown character: {}", c),
                    line_number,
                ))
            }
        }
    }
    Ok(tokens)
}
