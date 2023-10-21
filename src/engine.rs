use std::path::{Path, PathBuf};

use crate::{
    ast::{self, Expression},
    builtins,
    errors::{Error as MinicelError, ErrorKind as MinicelErrorKind, Result as MinicelResult},
    parser, tokenizer, utils,
};

/// The minicel-rs engine.
#[derive(Debug)]
pub struct Engine<'a> {
    /// The file
    pub file: std::path::PathBuf,
    /// Updated fields to be written back to the CSV file.
    /// (record, value)
    pub updated_records: Vec<(u64, Vec<String>)>,
    /// The csv lines
    // FIXME: This is bad, but this is not a product use project, so yeah
    pub lines: Vec<&'a str>,
    /// The count of csv rows 1-based
    rows: usize,
}

impl<'a> Engine<'a> {
    /// Creates a new engine from the given CSV file.
    pub fn new(csv_path: PathBuf, csv_str: &'a str) -> MinicelResult<Self> {
        let lines = csv_str.lines();
        Ok(Self {
            updated_records: Vec::new(),
            file: csv_path.to_path_buf(),
            // Minus the csv header
            rows: lines.clone().count() - 1,
            lines: csv_str.lines().collect(),
        })
    }

    /// Runs the given function call.
    #[allow(clippy::only_used_in_recursion)]
    pub fn function_call(
        &mut self,
        mut function_call: ast::FunctionCallExpression,
    ) -> MinicelResult<String> {
        log::info!("Running function call: {function_call:#?}");

        for arg in function_call.arguments.iter_mut() {
            if let Expression::FunctionCall(arg_function_call) = arg {
                log::debug!(
                    "Found function call in `{}` args: {arg_function_call:?}",
                    function_call.name
                );
                let value = self.function_call(arg_function_call.clone())?;
                *arg = utils::parse_string_to_expression(value);
            }
        }

        if let Some(builtin) = builtins::call_builtin(&function_call.name, function_call.arguments)
        {
            log::info!(
                "Running {} builtin function successfully",
                function_call.name
            );
            match builtin {
                Ok(value) => {
                    log::debug!("Builtin function returned: {value}");
                    Ok(value)
                }
                Err(error) => {
                    log::error!("Builtin function error: {error}");
                    Err(MinicelError::new(
                        MinicelErrorKind::Engine,
                        format!("Builtin function error: {error}"),
                        function_call.line_number,
                    ))
                }
            }
        } else {
            Err(MinicelError::new(
                MinicelErrorKind::Engine,
                format!("Unknown function {}", function_call.name),
                function_call.line_number,
            ))
        }
    }

    /// Executes the given field if it is a function call.
    pub fn execute_field(&mut self, field: String, line_number: usize) -> MinicelResult<String> {
        log::info!("Executing field \"{field}\" at line {line_number}");

        if field.starts_with('=') {
            log::info!("Field is a function call");

            let tokens = tokenizer::tokenize(field.trim_start_matches('=').trim(), line_number)?;
            log::debug!("Field tokens: {tokens:?}");
            let mut parser = parser::Parser::new(multipeek::multipeek(tokens.iter()), line_number);
            log::debug!("Field parser: {parser:#?}");
            let mut ast = parser.parse()?;

            log::info!("Executing child expressions");
            for expr in ast.mut_children() {
                log::debug!("Executing child expression: {expr:#?}");

                if let Expression::FunctionCall(function_call) = expr {
                    log::debug!("Child expression is a function call");
                    let value = self.function_call(function_call.clone())?;
                    *expr = Expression::String(value);
                } else if let Expression::Array(array) = expr {
                    log::debug!("Child expression is an array");
                    log::info!("Executing child expressions in array");
                    for element in array {
                        log::debug!("Executing child expression in array: {element:#?}");
                        if let Expression::FunctionCall(function_call) = element {
                            log::debug!("Child expression in array is a function call");
                            let value = self.function_call(function_call.clone())?;
                            *element = Expression::String(value);
                        } else if let Expression::Field { col, row, .. } = element {
                            log::debug!(
                                "Child expression in array is a field Col: {col}, Row: {row}"
                            );
                            let value = self.get_field(
                                utils::col_number_from_alpha(col),
                                *row,
                                line_number,
                            )?;
                            *element = utils::parse_string_to_expression(value);
                        }
                    }
                } else if let Expression::Field { col, row, .. } = expr {
                    log::info!("Child expression is a field Col: {col}, Row: {row}");

                    let value =
                        self.get_field(utils::col_number_from_alpha(col), *row, line_number)?;
                    *expr = utils::parse_string_to_expression(value);
                }
            }
            self.function_call(ast.function)
        } else {
            log::info!("Field is not a function call");
            Ok(field)
        }
    }

    /// Returns the record by row.
    pub fn get_record(&mut self, row: usize) -> MinicelResult<Vec<String>> {
        log::info!("Getting record Row: {row}");

        if row > self.rows {
            return Err(MinicelError::new(
                MinicelErrorKind::Engine,
                format!("Invalid row number {row}, the rows is {}", self.rows),
                row + 1,
            ));
        }

        let record = self
            .lines
            .get(row)
            .unwrap()
            .split(',')
            .map(|f| f.trim().to_owned())
            .collect::<Vec<String>>();
        log::debug!("Read record: {record:?} successfully");
        Ok(record)
    }

    /// Returns the field value by column and row.
    pub fn get_field(&mut self, col: usize, row: u64, line_number: usize) -> MinicelResult<String> {
        log::info!("Getting field Col: {col}, Row: {row}");

        let str_value =
            if let Some(updated_field) = self.updated_records.iter().find(|f| f.0 == row) {
                log::debug!(
                    "Found the record as an updated record: {:?}",
                    updated_field.1
                );
                if updated_field.1.len() < col {
                    return Err(MinicelError::new(
                        MinicelErrorKind::Engine,
                        format!(
                            "CSV error: Record {row} has only {} columns, cannot get column {col}",
                            updated_field.1.len(),
                        ),
                        line_number,
                    ));
                }
                log::debug!("Returning the updated field: {}", updated_field.1[col]);
                updated_field.1[col].clone()
            } else {
                log::info!("Getting the record from the CSV file");
                let field_line_number = (row + 1) as usize;

                let record = self.get_record(row as usize)?;
                if record.len() < col {
                    return Err(MinicelError::new(
                        MinicelErrorKind::Engine,
                        format!(
                            "CSV error: Record {row} has only {} columns, cannot get column {col}",
                            record.len(),
                        ),
                        field_line_number,
                    ));
                }

                log::debug!("Executing the field: {}", record[col]);
                let field = self.execute_field(record[col].trim().to_owned(), field_line_number)?;
                log::debug!("Returning the field: {}", field);
                field
            };

        Ok(str_value)
    }

    /// Update the given field value
    pub fn update_field(
        &mut self,
        col: usize,
        row: u64,
        value: String,
        line_number: usize,
    ) -> MinicelResult<()> {
        log::debug!("Updating field Col: {col}, Row: {row} with value: {value}");

        let static_record = self.get_record(row as usize)?;
        if static_record.len() <= col {
            return Err(MinicelError::new(
                MinicelErrorKind::Engine,
                format!(
                    "CSV error: Record {row} has only {} columns, cannot update column {}",
                    static_record.len(),
                    col + 1
                ),
                line_number,
            ));
        }

        let mut new_record = static_record.clone();
        new_record[col] = value;

        // If the record is already updated, update the updated record.
        if let Some(old_idx) = self.updated_records.iter().position(|(r, _)| r == &row) {
            let (_, old_record) = self.updated_records.remove(old_idx);
            self.updated_records.push((
                row,
                utils::compare_records(static_record, old_record, new_record),
            ));
        } else {
            self.updated_records.push((row, new_record));
        }

        Ok(())
    }

    /// Runs the engine.
    pub fn run(&mut self, out_file: &Path) -> MinicelResult<()> {
        let mut writer = csv::Writer::from_path(out_file).map_err(|err| {
            MinicelError::new(
                MinicelErrorKind::Engine,
                format!("Write CSV file error `{}`", err),
                0,
            )
        })?;

        for (row, record) in self.lines.clone().iter().enumerate() {
            if record.is_empty() {
                continue;
            }
            for (col, field) in record.split(',').enumerate() {
                let execution_field = self.execute_field(field.trim().to_string(), row + 1)?;
                if execution_field != field {
                    self.update_field(col, row as u64, execution_field, row + 1)?;
                }
            }
            if let Some((_, updated_record)) = self
                .updated_records
                .iter()
                .find(|(r, _)| r == &(row as u64))
            {
                writer.write_record(updated_record).map_err(|err| {
                    MinicelError::new(
                        MinicelErrorKind::Engine,
                        format!("Write CSV record error `{}`", err),
                        row + 1,
                    )
                })?;
            } else {
                writer.write_record(record.split(',')).map_err(|err| {
                    MinicelError::new(
                        MinicelErrorKind::Engine,
                        format!("Write CSV record error `{}`", err),
                        row + 1,
                    )
                })?;
            }
            if row % 100 == 0 {
                writer.flush().map_err(|err| {
                    MinicelError::new(
                        MinicelErrorKind::Engine,
                        format!("Flush CSV file error `{}`", err),
                        row + 1,
                    )
                })?;
            }
        }
        writer.flush().map_err(|err| {
            MinicelError::new(
                MinicelErrorKind::Engine,
                format!("Flush CSV file error `{}`", err),
                0,
            )
        })?;

        Ok(())
    }
}
