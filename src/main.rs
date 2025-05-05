use std::{fs, process};

use chumsky::prelude::*;

mod ast;
mod matcher;
mod parser;
mod runner;

use parser::parser;
use runner::Runner;

/// # Panics
/// on invalid input
pub fn main() {
    let Some(path) = std::env::args().nth(1) else {
        eprintln!("No file provided");
        process::exit(1);
    };

    let Ok(mut input) = fs::read_to_string(path) else {
        eprintln!("Failed to read file");
        process::exit(1);
    };

    input = input.replace('\r', "");

    let parse = parser().parse(&input);
    let result = parse.into_result().unwrap();

    let runner = Runner::new(result);
    match runner.run() {
        Ok(output) => println!("{output}"),
        Err(err) => eprintln!("Error: {err}"),
    }
}
