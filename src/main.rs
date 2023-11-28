mod lexer;
mod token;

use std::fs;
use std::process;

fn main() {
    let program: String = match fs::read_to_string("program.asm") {
        Ok(s) => s,
        Err(e) => {
            panic!("{}", e)
        }
    };
    let mut prog = match AsmParser::parse(Rule::PROGRAM, program.as_str()) {
        Ok(prog) => prog,
        Err(e) => match e.variant {
            ErrorVariant::ParsingError {
                positives,
                negatives,
            } => {
                println!("parsing error {:?} {:?}", positives, negatives);
                process::exit(1);
            }
            ErrorVariant::CustomError { message } => {
                println!("custom error {}", message);
                process::exit(1);
            }
        },
    };
    let t = prog.next().unwrap();
}
