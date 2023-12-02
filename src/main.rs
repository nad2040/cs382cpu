mod lexer;
mod parser;
mod token;

use lexer::Lexer;
use parser::Parser;
use std::fs;

use log::{debug, error};

fn main() {
    env_logger::init();
    let program: String = match fs::read_to_string("program.asm") {
        Ok(s) => s,
        Err(e) => {
            error!("{}", e);
            std::process::exit(1);
        }
    };
    let lexer = Lexer::new(program);
    lexer.emit();
    let parser = Parser::new(lexer.tokens);
    parser.emit();

    // let mut prog = match AsmParser::parse(Rule::PROGRAM, program.as_str()) {
    //     Ok(prog) => prog,
    //     Err(e) => match e.variant {
    //         ErrorVariant::ParsingError {
    //             positives,
    //             negatives,
    //         } => {
    //             println!("parsing error {:?} {:?}", positives, negatives);
    //             process::exit(1);
    //         }
    //         ErrorVariant::CustomError { message } => {
    //             println!("custom error {}", message);
    //             process::exit(1);
    //         }
    //     },
    // };
    // let t = prog.next().unwrap();
}
