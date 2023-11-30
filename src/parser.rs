use crate::token::{DataTypeDirective, SectionDirective, Token, TokenValue};
use log::error;
use std::{collections::HashMap, fmt::format};

pub enum RegOrAddr {
    Register(u8),
    Address(usize),
    Unresolved(Token),
}

pub enum Instruction {
    Halt,

    Add(u8, u8, RegOrAddr),
    Sub(u8, u8, RegOrAddr),
    Mul(u8, u8, RegOrAddr),
    Div(u8, u8, RegOrAddr),
    Mod(u8, u8, RegOrAddr),
    Asr(u8, u8, RegOrAddr),
    Lsl(u8, u8, RegOrAddr),

    And(u8, u8, RegOrAddr),
    Orr(u8, u8, RegOrAddr),
    Neg(u8, u8, RegOrAddr),

    Swap(u8, u8),

    Ld(u8, RegOrAddr),                  // rd, (rn | addr)
    LdMem(u8, bool, u8, u8, RegOrAddr), // num bytes, sign extension, rd, rn, (rm | addr)
    St(u8, u8, RegOrAddr),              // rd, rn, (rm | addr)
}

#[derive(Debug)]
pub enum Data {
    String(String),
    Char(char),
    Byte1(u8),
    Byte2(u16),
    Byte4(u32),
    Byte8(u64),
}

pub struct Parser {
    tokens: Vec<Token>,
    token_idx: usize,
    mapping: HashMap<Token, (usize, Option<usize>)>,
    constant_pool_offset: usize,
    data_section_offset: usize,
    text_section_offset: usize,
    instructions: Vec<Instruction>,
    data_section: Vec<u8>,
    constant_pool: Vec<u8>,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        let mut p = Self {
            tokens,
            token_idx: 0,
            mapping: HashMap::new(),
            constant_pool_offset: 0x0000,
            data_section_offset: 0x1000,
            text_section_offset: 0x8000,
            instructions: Vec::new(),
            data_section: Vec::new(),
            constant_pool: Vec::new(),
        };
        p.parse();
        p
    }

    fn is_at_end(&self) -> bool {
        self.token_idx >= self.tokens.len()
    }

    fn peek(&self) -> Token {
        self.tokens[self.token_idx].clone()
    }

    fn increment_position(&mut self, n: usize) {
        self.token_idx += n
    }

    fn error(&self, message: String, token: Token) {
        error!("{} at {}:{}", message, token.loc.line, token.loc.col);
        std::process::exit(1);
    }

    pub fn parse(&mut self) {
        while !self.is_at_end() {
            let token = self.peek();
            // for now, just skip tokens until a section directive is found. then parse that section
            match &token.value {
                TokenValue::SectionDirective(section_type) => match section_type {
                    SectionDirective::Data => self.parse_data_section(),
                    SectionDirective::Text => self.parse_text_section(),
                },
                _ => self.increment_position(1),
            }
        }
        println!("{:?}", self.mapping);
        // now do second pass and resolve all unresolved labels.
        // if some label token is not in the hashmap already, we have an undefined label!
        self.resolve_labels();
    }

    fn parse_text_section(&mut self) {
        // keep track of constants, this should immediately resolve, and we can easily calculate
        // relative offset.
    }

    fn parse_instruction(&mut self) {
        // keep track of labels and calculate relative offset
    }

    fn parse_data_section(&mut self) {
        // also keep track of label definitions. we don't allow label usages in the data section
        self.increment_position(1);
        while !self.is_at_end() {
            let t = self.peek();
            match t.value.clone() {
                TokenValue::SectionDirective(_) => break,
                TokenValue::Eof => break,
                TokenValue::Whitespace | TokenValue::Newline => self.increment_position(1),
                TokenValue::LabelDef(_) => {
                    self.increment_position(1);
                    self.mapping.insert(t, (self.data_section_offset, None));
                }
                TokenValue::DataTypeDirective(datatype) => self.parse_datatype_directive(datatype),
                _ => self.error(format!("Unrecognized token {:?}", t.value), t),
            }
        }
    }

    fn parse_datatype_directive(&mut self, datatype: DataTypeDirective) {
        // maybe allow char for the ._b directives.
        let mut data = Vec::new();
        self.increment_position(1);
        let mut is_done = false;
        while !self.is_at_end() {
            let t = self.peek();
            match (t.value.clone(), datatype.clone()) {
                (TokenValue::Whitespace, _) => {
                    self.increment_position(1);
                    continue;
                }
                (TokenValue::DataTypeDirective(_), _) => break,
                (
                    TokenValue::Imm(num),
                    DataTypeDirective::Byte1
                    | DataTypeDirective::Byte2
                    | DataTypeDirective::Byte4
                    | DataTypeDirective::Byte8,
                ) => match datatype {
                    DataTypeDirective::Byte1 => {
                        if num > u8::MAX as u64 {
                            self.error(
                                format!("Immediate is too big for .1b directive {:?}", t.value),
                                t,
                            );
                        }
                        data.push(Data::Byte1(num as u8));
                    }
                    DataTypeDirective::Byte2 => {
                        if num > u16::MAX as u64 {
                            self.error(
                                format!("Immediate is too big for .1b directive {:?}", t.value),
                                t,
                            );
                        }
                        data.push(Data::Byte2(num as u16));
                    }
                    DataTypeDirective::Byte4 => {
                        if num > u32::MAX as u64 {
                            self.error(
                                format!("Immediate is too big for .1b directive {:?}", t.value),
                                t,
                            );
                        }
                        data.push(Data::Byte4(num as u32));
                    }
                    DataTypeDirective::Byte8 => data.push(Data::Byte8(num)),
                    _ => self.error(
                        "Illegal state when parsing list of immediates".to_string(),
                        t,
                    ),
                },
                (TokenValue::Char(ch), DataTypeDirective::Char) => {
                    data.push(Data::Char(ch));
                }
                (TokenValue::String(str), DataTypeDirective::String) => {
                    data.push(Data::String(str));
                }
                (_, _) => self.error(format!("Unexpected token {:?}", t.value), t),
            }
            self.increment_position(1);
            while !self.is_at_end() {
                let sep = self.peek();
                match sep.value {
                    TokenValue::Comma => {
                        self.increment_position(1);
                        break;
                    }
                    TokenValue::Whitespace => {
                        self.increment_position(1);
                        continue;
                    }
                    TokenValue::Newline => {
                        is_done = true;
                        break;
                    }
                    _ => self.error(format!("Unexpected token {:?}", sep.value), sep),
                }
            }
            if is_done {
                break;
            }
        }
        println!("{:?}", data);
        for d in data.into_iter() {
            match d {
                Data::String(str) => {
                    self.data_section.append(&mut str.as_bytes().to_vec());
                    self.data_section.push(0);
                    self.data_section_offset += str.len() + 1; // +1 for null terminator
                }
                Data::Char(ch) => {
                    self.data_section.push(ch as u8);
                    self.data_section_offset += 1;
                }
                Data::Byte1(byte1) => {
                    self.data_section.push(byte1);
                    self.data_section_offset += 1;
                }
                Data::Byte2(byte2) => {
                    self.data_section.append(&mut byte2.to_le_bytes().to_vec());
                    self.data_section_offset += 2;
                }
                Data::Byte4(byte4) => {
                    self.data_section.append(&mut byte4.to_le_bytes().to_vec());
                    self.data_section_offset += 4;
                }
                Data::Byte8(byte8) => {
                    self.data_section.append(&mut byte8.to_le_bytes().to_vec());
                    self.data_section_offset += 8;
                }
            }
        }
        println!("{:?}", self.data_section);
    }

    fn resolve_labels(&mut self) {
        // resolve labels. if label not the hashmap, we have an error.
        todo!()
    }
}
