use std::{ffi::OsStr, path::Path, str::FromStr};

use bigdecimal::BigDecimal;

use crate::ast::Expression;

/// Returns the col number from the alphabet. e.g. `A` -> `1`, `B` -> `2`, `AA` -> `27`
pub fn col_number_from_alpha(alpha: &str) -> usize {
    log::info!("Converting alpha to col number: {alpha}");

    let mut col = 0;
    for c in alpha.chars() {
        col *= 26;
        col += (c as u8 - b'A') as usize + 1;
    }
    log::debug!(
        "Converted alpha to col number (Starting from 0): {}",
        col - 1
    );
    col - 1
}

/// Compare tow record updates and returns the updated fields.
/// e.g.
/// Static: ["=print(A1)", "=print(B2)", "=print(C3)", "=print(D4)", "=print(E5)"]
/// Old: ["=print(A1)", "32", "=print(C3)", "=print(D4)", "=print(E5)"]
/// New: ["=print(A1)", "=print(B2)", "Male", "=print(D4)", "=print(E5)"]
/// Output: ["=print(A1)", "32", "Male", "=print(D4)", "=print(E5)"]
///
/// Static: ["=print(A1)", "=print(B2)", "=print(C3)", "=print(D4)", "=print(E5)"]
/// Old: ["=print(A1)", "32", "Male", "=print(D4)", "=print(E5)"[
/// New: ["=print(A1)", "=print(B2)", "=print(C3)", "USA", "=print(E5)"]
/// Output: ["=print(A1)", "32", "Male", "USA", "=print(E5)"]
///
/// The function will look at each field in the new record, if the field is not equal to the static record and the old record, it will return the field.
/// if the field is equal to the static field and not equal to the old field, will return the old field.
pub fn compare_records(
    static_record: Vec<String>,
    old_record: Vec<String>,
    new_record: Vec<String>,
) -> Vec<String> {
    static_record
        .iter()
        .zip(new_record.iter())
        .zip(old_record.iter())
        .map(|((static_field, old_field), new_filed)| {
            let (static_field, old_field, new_filed) =
                (static_field.trim(), old_field.trim(), new_filed.trim());
            if new_filed == static_field && new_filed != old_field {
                old_field.to_string()
            } else {
                new_filed.to_string()
            }
        })
        .collect()
}

/// CSV file path check
pub fn check_csv_file_path(path: &Path, exists: bool) -> Result<(), String> {
    if exists && !path.exists() {
        Err(format!("{} does not exist", path.display()))
    } else if path.exists() && !path.is_file() {
        Err(format!("{} is not a file", path.display()))
    } else if matches!(path.extension(), Some(e) if e.to_ascii_lowercase() != OsStr::new("csv")) {
        Err(format!("{} is not a CSV file", path.display()))
    } else if !exists && !path.exists() {
        std::fs::File::create(path).map_err(|e| e.to_string())?;
        Ok(())
    } else {
        Ok(())
    }
}

/// Parse the string if it is a [`Expression::Number`] or [`Expression::Float`] or [`Expression::String`]
pub fn parse_string_to_expression(string: String) -> Expression {
    if let Ok(number) = BigDecimal::from_str(&string) {
        Expression::Number(number)
    } else {
        Expression::String(string.to_string())
    }
}
