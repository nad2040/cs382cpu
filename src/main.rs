extern crate pest;
#[macro_use]
extern crate pest_derive;

#[derive(Parser)]
#[grammar = "grammar.pest"]
struct AsmParser;

use pest::Parser;

use std::fs;

fn main() {
    let program: String = match fs::read_to_string("program.asm") {
        Ok(s) => s,
        Err(e) => {
            panic!("{}", e)
        }
    };
    let tokens = match AsmParser::parse(Rule::PROGRAM, program.as_str()) {
        Ok(tokens) => tokens,
        Err(e) => {
            panic!("{}", e)
        }
    };
    println!("{}", tokens);
    for t in tokens.flatten().tokens() {
        println!("{:?}",t);
    }
}
