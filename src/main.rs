use std::{fs, process::exit};

mod ast;
mod builtins;
mod engine;
mod errors;
mod parser;
mod tokenizer;
mod utils;

const fn help_message() -> &'static str {
    "Usage: minicel-rs <input.csv> <out.csv>"
}

fn main() {
    let _args: Vec<String> = std::env::args().collect();
    pretty_env_logger::init();

    let args = vec![".".to_owned(), "test.csv".to_owned(), "out.csv".to_owned()];

    if args.len() != 3 {
        println!("{}", help_message());
        exit(1);
    }
    let input_path = std::path::Path::new(&args[1]);
    let output_path = std::path::Path::new(&args[2]);

    if let Err(error) = utils::check_csv_file_path(input_path, true) {
        println!("{error}");
        exit(1);
    }
    if let Err(error) = utils::check_csv_file_path(output_path, false) {
        println!("{error}");
        exit(1);
    }

    let Ok(csv_content) = fs::read_to_string(input_path) else {
        println!("IO error: Cannot read the input file");
        exit(1);
    };

    match engine::Engine::new(input_path.to_path_buf(), &csv_content) {
        Ok(mut engine) => {
            if let Err(err) = engine.run(output_path) {
                println!("{err}");
                exit(1);
            }
        }
        Err(err) => {
            println!("{err}");
            exit(1)
        }
    }
}
