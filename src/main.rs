mod lexer;
mod parser;
mod token;
mod txtfilegen;

use lexer::Lexer;
use parser::Parser;
use std::fs;

use log::error;

fn main() {
    env_logger::init();
    let args: Vec<String> = std::env::args().collect();
    // println!("{:?}", args);
    if args.len() < 2 {
        error!("Missing source file!\nUsage: ./target/release/cs382cpu <filename>");
        std::process::exit(1);
    }
    let program: String = match fs::read_to_string(&args[1]) {
        Ok(s) => s,
        Err(e) => {
            error!("{}", e);
            std::process::exit(1);
        }
    };
    let lexer = Lexer::new(program);
    // lexer.emit();
    let parser = Parser::new(lexer.tokens);
    // parser.emit();
    let program_name = std::path::Path::new(&args[1])
        .file_stem()
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
    txtfilegen::generate_files(
        program_name,
        &parser.constant_pool,
        &parser.data_section,
        &parser.instructions,
    );
}
