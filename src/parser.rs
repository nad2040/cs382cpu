use crate::token::{DataTypeDirective, SectionDirective, Token, TokenValue};
use log::{debug, error};
use std::collections::HashMap;

pub const CONSTANT_POOL_OFFSET: usize = 0x0000;
pub const DATA_OFFSET: usize = 0x40;
pub const TEXT_OFFSET: usize = 0x80;
pub const FILE_LIMIT: usize = 0x100;

#[derive(Debug)]
pub enum RegImmAddr {
    Register(u8),
    Imm(i16),
    Address(i16),                     // PC-relative offset
    Unresolved(String, usize, usize), // label name, current PC, current constant_pool_offset
}

#[derive(Debug)]
pub enum Instruction {
    Halt,

    Add(u8, u8, RegImmAddr),
    Sub(u8, u8, RegImmAddr),
    Mul(u8, u8, RegImmAddr),
    Div(u8, u8, RegImmAddr),
    Mod(u8, u8, RegImmAddr),
    Asr(u8, u8, RegImmAddr),
    Lsl(u8, u8, RegImmAddr),

    And(u8, u8, RegImmAddr),
    Orr(u8, u8, RegImmAddr),
    Neg(u8, RegImmAddr),

    Swap(u8, u8),

    Ld(u8, RegImmAddr),                  // rd, (rn | addr)
    LdMem(u8, bool, u8, u8, RegImmAddr), // num bytes, sign extension, rd, rn, (rm | addr)
    St(u8, u8, u8, RegImmAddr),          // num bytes, rd, rn, (rm | addr)

    B(RegImmAddr),
    CBZ(u8, RegImmAddr),
    CBNZ(u8, RegImmAddr),
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
    mapping: HashMap<String, usize>,
    constant_pool_offset: usize,
    data_section_offset: usize,
    text_section_offset: usize,
    pub instructions: Vec<Instruction>,
    pub data_section: Vec<u8>,
    pub constant_pool: Vec<u8>,
}

#[allow(dead_code)]
impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        let mut p = Self {
            tokens,
            token_idx: 0,
            mapping: HashMap::new(),
            constant_pool_offset: CONSTANT_POOL_OFFSET,
            data_section_offset: DATA_OFFSET,
            text_section_offset: TEXT_OFFSET,
            instructions: Vec::new(),
            data_section: Vec::new(),
            constant_pool: Vec::new(),
        };
        p.parse();
        p
    }

    pub fn emit(&self) {
        for instr in &self.instructions {
            println!("{:?}", instr);
        }
        println!("{:?}", self.data_section);
        println!("{:?}", self.mapping);
    }

    fn is_at_end(&self) -> bool {
        self.token_idx >= self.tokens.len()
    }

    fn peek(&self) -> Token {
        self.tokens[self.token_idx].clone()
    }

    fn skip_whitespace(&mut self) {
        while !self.is_at_end() && self.peek().value == TokenValue::Whitespace {
            self.increment_position(1);
        }
    }

    fn expect_comma(&mut self) {
        if self.peek().value != TokenValue::Comma {
            self.errtok(
                format!("Expected comma but found {:?}", self.peek().value),
                self.peek(),
            )
        }
        self.increment_position(1);
    }

    fn increment_position(&mut self, n: usize) {
        self.token_idx += n
    }

    fn errtok(&self, message: String, token: Token) {
        error!("{} at {}:{}", message, token.loc.line, token.loc.col);
        std::process::exit(1);
    }
    fn errmsg(&self, message: String) {
        error!("{}", message);
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
        debug!("{:?}", self.instructions);
        // now do second pass and resolve all unresolved labels.
        // if some label token is not in the hashmap already, we have an undefined label!
        self.resolve_labels();
        debug!("{:?}", self.mapping);
        debug!("{:?}", self.constant_pool);
        debug!("{:?}", self.data_section);
        debug!("{:?}", self.instructions);
    }

    fn parse_text_section(&mut self) {
        // keep track of constants, this should immediately resolve, and we can easily calculate
        // relative offset.
        self.increment_position(1);
        while !self.is_at_end() {
            let t = self.peek();
            match t.value.clone() {
                TokenValue::SectionDirective(_) | TokenValue::Eof => break,
                TokenValue::Whitespace | TokenValue::Newline => self.increment_position(1),
                TokenValue::LabelDef(label) => {
                    self.increment_position(1);
                    self.mapping.insert(label, self.text_section_offset);
                }
                TokenValue::Add
                | TokenValue::Sub
                | TokenValue::Mul
                | TokenValue::Div
                | TokenValue::Mod
                | TokenValue::Asr
                | TokenValue::Lsl
                | TokenValue::And
                | TokenValue::Orr => self.parse_reg_reg_immreg_instruction(t.value),
                TokenValue::Neg => self.parse_neg_instruction(),
                TokenValue::Swap => self.parse_swap_instruction(),
                TokenValue::Ld(num_bytes, sign_extension) => {
                    self.parse_ld_instruction(num_bytes, sign_extension);
                }
                TokenValue::St(num_bytes) => self.parse_st_instruction(num_bytes),
                TokenValue::Halt => self.parse_halt_instruction(),
                TokenValue::B => self.parse_branch_instruction(),
                TokenValue::CBZ | TokenValue::CBNZ => self.parse_cbz_cbnz_instruction(t.value),
                _ => self.errtok(format!("Unexpected token {:?}", t.value), t),
            }
        }
    }

    fn parse_halt_instruction(&mut self) {
        self.instructions.push(Instruction::Halt);
        self.text_section_offset += 4;
        self.increment_position(1);
        self.skip_whitespace();
        match self.peek().value {
            TokenValue::Newline => self.increment_position(1),
            _ => self.errtok(
                format!("Unexpected token {:?}", self.peek().value),
                self.peek(),
            ),
        }
    }

    fn parse_branch_instruction(&mut self) {
        self.increment_position(1);
        self.skip_whitespace();
        match self.peek().value {
            TokenValue::Label(label) => {
                self.instructions.push(Instruction::B(
                    RegImmAddr::Unresolved(
                        label,
                        self.text_section_offset,
                        self.constant_pool_offset,
                    ), // current PC. calculate
                       // relative offset later
                ));
            }
            _ => self.errtok(
                format!(
                    "Expected label token for branch instruction but found {:?}",
                    self.peek().value
                ),
                self.peek(),
            ),
        }
        self.text_section_offset += 4;
        self.increment_position(1);
        self.skip_whitespace();
        match self.peek().value {
            TokenValue::Newline => self.increment_position(1),
            _ => self.errtok(
                format!("Unexpected token {:?}", self.peek().value),
                self.peek(),
            ),
        }
    }
    fn parse_cbz_cbnz_instruction(&mut self, cb_instruction: TokenValue) {
        let mut reg: Option<u8> = None;
        self.increment_position(1);
        self.skip_whitespace();
        match self.peek().value {
            TokenValue::Register(register_num) => reg = Some(register_num),
            _ => self.errtok("Expected a register to check".to_string(), self.peek()),
        }
        self.increment_position(1);
        self.skip_whitespace();
        self.expect_comma();
        self.skip_whitespace();
        match (self.peek().value, cb_instruction) {
            (TokenValue::Label(label), TokenValue::CBZ) => {
                self.instructions.push(Instruction::CBZ(
                    reg.unwrap(),
                    RegImmAddr::Unresolved(
                        label,
                        self.text_section_offset,
                        self.constant_pool_offset,
                    ), // current PC. calculate
                       // relative offset later
                ));
            }
            (TokenValue::Label(label), TokenValue::CBNZ) => {
                self.instructions.push(Instruction::CBNZ(
                    reg.unwrap(),
                    RegImmAddr::Unresolved(
                        label,
                        self.text_section_offset,
                        self.constant_pool_offset,
                    ), // current PC. calculate
                       // relative offset later
                ));
            }
            (_, _) => self.errtok(
                format!(
                    "Expected label token for branch instruction but found {:?}",
                    self.peek().value
                ),
                self.peek(),
            ),
        }
        self.text_section_offset += 4;
        self.increment_position(1);
        self.skip_whitespace();
        match self.peek().value {
            TokenValue::Newline => self.increment_position(1),
            _ => self.errtok(
                format!("Unexpected token {:?}", self.peek().value),
                self.peek(),
            ),
        }
    }

    fn parse_neg_instruction(&mut self) {
        let mut dst: Option<u8> = None;
        let mut src: Option<RegImmAddr> = None;
        self.increment_position(1);
        self.skip_whitespace();
        match self.peek().value {
            TokenValue::Register(register_num) => dst = Some(register_num),
            _ => self.errtok("Expected a register".to_string(), self.peek()),
        }
        self.increment_position(1);
        self.skip_whitespace();
        self.expect_comma();
        self.skip_whitespace();
        match self.peek().value {
            TokenValue::Register(register_num) => src = Some(RegImmAddr::Register(register_num)),
            TokenValue::Imm(imm) => {
                if imm > u16::MAX as u64 {
                    self.errtok(
                        format!("Immediate is too big for instruction {:?}", TokenValue::Neg),
                        self.peek(),
                    );
                }
                src = Some(RegImmAddr::Imm(imm as i16));
            }
            TokenValue::Char(ch) => {
                src = Some(RegImmAddr::Imm(ch as i16));
            }
            _ => self.errtok("Expected a register or immediate".to_string(), self.peek()),
        }
        let dst = dst.unwrap();
        let src = src.unwrap();
        self.instructions.push(Instruction::Neg(dst, src));
        self.text_section_offset += 4;
        self.increment_position(1);
        self.skip_whitespace();
        match self.peek().value {
            TokenValue::Newline => self.increment_position(1),
            _ => self.errtok(
                format!("Unexpected token {:?}", self.peek().value),
                self.peek(),
            ),
        }
    }
    fn parse_swap_instruction(&mut self) {
        let mut reg1: Option<u8> = None;
        let mut reg2: Option<u8> = None;
        self.increment_position(1);
        self.skip_whitespace();
        match self.peek().value {
            TokenValue::Register(register_num) => reg1 = Some(register_num),
            _ => self.errtok("Expected a register".to_string(), self.peek()),
        }
        self.increment_position(1);
        self.skip_whitespace();
        self.expect_comma();
        self.skip_whitespace();
        match self.peek().value {
            TokenValue::Register(register_num) => reg2 = Some(register_num),
            _ => self.errtok("Expected a register".to_string(), self.peek()),
        }
        let reg1 = reg1.unwrap();
        let reg2 = reg2.unwrap();
        self.instructions.push(Instruction::Swap(reg1, reg2));
        self.text_section_offset += 4;
        self.increment_position(1);
        self.skip_whitespace();
        match self.peek().value {
            TokenValue::Newline => self.increment_position(1),
            _ => self.errtok(
                format!("Unexpected token {:?}", self.peek().value),
                self.peek(),
            ),
        }
    }
    fn parse_reg_reg_immreg_instruction(&mut self, instruction_op: TokenValue) {
        let mut dst: Option<u8> = None;
        let mut src1: Option<u8> = None;
        let mut src2: Option<RegImmAddr> = None;
        self.increment_position(1);
        self.skip_whitespace();
        match self.peek().value {
            TokenValue::Register(register_num) => dst = Some(register_num),
            _ => self.errtok("Expected a register".to_string(), self.peek()),
        }
        self.increment_position(1);
        self.skip_whitespace();
        self.expect_comma();
        self.skip_whitespace();
        match self.peek().value {
            TokenValue::Register(register_num) => src1 = Some(register_num),
            _ => self.errtok("Expected a register".to_string(), self.peek()),
        }
        self.increment_position(1);
        self.skip_whitespace();
        self.expect_comma();
        self.skip_whitespace();
        match self.peek().value {
            TokenValue::Register(register_num) => src2 = Some(RegImmAddr::Register(register_num)),
            TokenValue::Imm(imm) => {
                if imm > u16::MAX as u64 {
                    self.errtok(
                        format!("Immediate is too big for instruction {:?}", instruction_op),
                        self.peek(),
                    );
                }
                src2 = Some(RegImmAddr::Imm(imm as i16));
            }
            TokenValue::Char(ch) => {
                src2 = Some(RegImmAddr::Imm(ch as i16));
            }
            _ => self.errtok("Expected a register or immediate".to_string(), self.peek()),
        }
        let dst = dst.unwrap();
        let src1 = src1.unwrap();
        let src2 = src2.unwrap();
        match instruction_op {
            TokenValue::Add => self.instructions.push(Instruction::Add(dst, src1, src2)),
            TokenValue::Sub => self.instructions.push(Instruction::Sub(dst, src1, src2)),
            TokenValue::Mul => self.instructions.push(Instruction::Mul(dst, src1, src2)),
            TokenValue::Div => self.instructions.push(Instruction::Div(dst, src1, src2)),
            TokenValue::Mod => self.instructions.push(Instruction::Mod(dst, src1, src2)),
            TokenValue::Asr => self.instructions.push(Instruction::Asr(dst, src1, src2)),
            TokenValue::Lsl => self.instructions.push(Instruction::Lsl(dst, src1, src2)),
            TokenValue::And => self.instructions.push(Instruction::And(dst, src1, src2)),
            TokenValue::Orr => self.instructions.push(Instruction::Orr(dst, src1, src2)),
            _ => self.errtok(
                format!(
                    "Illegal state when parsing {:?} instruction",
                    instruction_op
                ),
                self.peek(),
            ),
        }
        self.text_section_offset += 4;
        self.increment_position(1);
        self.skip_whitespace();
        match self.peek().value {
            TokenValue::Newline => self.increment_position(1),
            _ => self.errtok(
                format!("Unexpected token {:?}", self.peek().value),
                self.peek(),
            ),
        }
    }

    fn parse_ld_instruction(&mut self, num_bytes: u8, sign_extension: bool) {
        self.increment_position(1);
        self.skip_whitespace();
        let mut dst: Option<u8> = None;
        match self.peek().value {
            TokenValue::Register(register_num) => dst = Some(register_num),
            _ => self.errtok("Expected a register".to_string(), self.peek()),
        }
        self.increment_position(1);
        self.skip_whitespace();
        self.expect_comma();
        self.skip_whitespace();
        match (self.peek().value, num_bytes) {
            (TokenValue::Register(register_num), 8) => {
                self.instructions.push(Instruction::Ld(
                    dst.unwrap(),
                    RegImmAddr::Register(register_num),
                ));
                self.increment_position(1);
            }
            (TokenValue::Label(label), 8) => {
                self.instructions.push(Instruction::Ld(
                    dst.unwrap(),
                    RegImmAddr::Unresolved(
                        label,
                        self.text_section_offset,
                        self.constant_pool_offset,
                    ), // current PC. calculate
                       // relative offset later
                ));
                self.constant_pool
                    .append(&mut (0 as u64).to_le_bytes().to_vec());
                self.constant_pool_offset += 8;
                self.increment_position(1);
            }
            (TokenValue::Imm(imm), 8) => {
                self.instructions.push(Instruction::Ld(
                    dst.unwrap(),
                    RegImmAddr::Address(
                        (self.constant_pool_offset as isize - self.text_section_offset as isize)
                            as i16,
                    ),
                ));
                self.constant_pool.append(&mut imm.to_le_bytes().to_vec());
                self.constant_pool_offset += 8;
                self.increment_position(1);
            }
            (TokenValue::Char(ch), 8) => {
                self.instructions.push(Instruction::Ld(
                    dst.unwrap(),
                    RegImmAddr::Address(
                        (self.constant_pool_offset as isize - self.text_section_offset as isize)
                            as i16,
                    ),
                ));
                self.constant_pool
                    .append(&mut (ch as u64).to_le_bytes().to_vec());
                self.constant_pool_offset += 8;
                self.increment_position(1);
            }
            (TokenValue::LBracket, _) => {
                let (addr_reg, offset) = self.parse_memory_access();
                self.instructions.push(Instruction::LdMem(
                    num_bytes,
                    sign_extension,
                    dst.unwrap(),
                    addr_reg,
                    offset,
                ))
            }
            (_, _) => self.errtok("Invalid LD instruction syntax".to_string(), self.peek()),
        }
        self.text_section_offset += 4;
        self.skip_whitespace();
        match self.peek().value {
            TokenValue::Newline => self.increment_position(1),
            _ => self.errtok(
                format!("Unexpected token {:?}", self.peek().value),
                self.peek(),
            ),
        }
    }
    fn parse_st_instruction(&mut self, num_bytes: u8) {
        self.increment_position(1);
        self.skip_whitespace();
        let mut src: Option<u8> = None;
        match self.peek().value {
            TokenValue::Register(register_num) => src = Some(register_num),
            _ => self.errtok("Expected a register".to_string(), self.peek()),
        }
        self.increment_position(1);
        self.skip_whitespace();
        self.expect_comma();
        self.skip_whitespace();
        match self.peek().value {
            TokenValue::LBracket => {
                let (addr_reg, offset) = self.parse_memory_access();
                self.instructions
                    .push(Instruction::St(num_bytes, src.unwrap(), addr_reg, offset))
            }
            _ => self.errtok("Invalid ST instruction syntax".to_string(), self.peek()),
        }
        self.text_section_offset += 4;
        self.skip_whitespace();
        match self.peek().value {
            TokenValue::Newline => self.increment_position(1),
            _ => self.errtok(
                format!("Unexpected token {:?}", self.peek().value),
                self.peek(),
            ),
        }
    }

    fn parse_memory_access(&mut self) -> (u8, RegImmAddr) {
        self.increment_position(1); // consume the '['
        self.skip_whitespace();
        let mut addr_reg: Option<u8> = None;
        let mut offset: Option<RegImmAddr> = None;
        match self.peek().value {
            TokenValue::Register(register_num) => addr_reg = Some(register_num),
            _ => self.errtok("Expected a register".to_string(), self.peek()),
        }
        self.increment_position(1);
        self.skip_whitespace();
        match self.peek().value {
            TokenValue::Comma => {
                self.increment_position(1);
                self.skip_whitespace();
                match self.peek().value {
                    TokenValue::Register(register_num) => {
                        offset = Some(RegImmAddr::Register(register_num));
                    }
                    TokenValue::Imm(imm) => {
                        if imm > u16::MAX as u64 {
                            self.errtok(
                                "Immediate for offset is too big for LD/ST instruction".to_string(),
                                self.peek(),
                            );
                        }
                        offset = Some(RegImmAddr::Imm(imm as i16));
                    }
                    TokenValue::Char(ch) => {
                        // I'm leaving this as a quirk for our assembly.
                        offset = Some(RegImmAddr::Imm(ch as i16));
                    }
                    _ => self.errtok("Expected a register or immediate".to_string(), self.peek()),
                }
                self.increment_position(1);
            }
            _ => (),
        }
        self.skip_whitespace();
        match self.peek().value {
            TokenValue::RBracket => self.increment_position(1),
            _ => self.errtok("Expected ']'".to_string(), self.peek()),
        }

        match offset {
            Some(offset) => (addr_reg.unwrap(), offset),
            None => (addr_reg.unwrap(), RegImmAddr::Imm(0)),
        }
    }

    fn parse_data_section(&mut self) {
        // also keep track of label definitions. we don't allow label usages in the data section
        self.increment_position(1);
        while !self.is_at_end() {
            let t = self.peek();
            match t.value.clone() {
                TokenValue::SectionDirective(_) | TokenValue::Eof => break,
                TokenValue::Whitespace | TokenValue::Newline => self.increment_position(1),
                TokenValue::LabelDef(label) => {
                    self.increment_position(1);
                    self.mapping.insert(label, self.data_section_offset);
                }
                TokenValue::DataTypeDirective(datatype) => self.parse_datatype_directive(datatype),
                _ => self.errtok(format!("Unexpected token {:?}", t.value), t),
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
                            self.errtok(
                                format!("Immediate is too big for .1b directive {:?}", t.value),
                                t,
                            );
                        }
                        data.push(Data::Byte1(num as u8));
                    }
                    DataTypeDirective::Byte2 => {
                        if num > u16::MAX as u64 {
                            self.errtok(
                                format!("Immediate is too big for .1b directive {:?}", t.value),
                                t,
                            );
                        }
                        data.push(Data::Byte2(num as u16));
                    }
                    DataTypeDirective::Byte4 => {
                        if num > u32::MAX as u64 {
                            self.errtok(
                                format!("Immediate is too big for .1b directive {:?}", t.value),
                                t,
                            );
                        }
                        data.push(Data::Byte4(num as u32));
                    }
                    DataTypeDirective::Byte8 => data.push(Data::Byte8(num)),
                    _ => self.errtok(
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
                (_, _) => self.errtok(format!("Unexpected token {:?}", t.value), t),
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
                    _ => self.errtok(format!("Unexpected token {:?}", sep.value), sep),
                }
            }
            if is_done {
                break;
            }
        }
        debug!("{:?}", data);
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
        debug!("{:?}", self.data_section);
    }

    fn resolve_labels(&mut self) {
        // resolve labels. if label not the hashmap, we have an error.
        for i in 0..self.instructions.len() {
            match &self.instructions[i] {
                Instruction::Ld(dst, RegImmAddr::Unresolved(label, pc, const_pool)) => {
                    // since for LD reg, label we need the physical label to be loaded from memory,
                    // I need to keep track of where the label should live when parsing it and
                    // leave space for it when it gets resolved.
                    match self.mapping.get(label) {
                        Some(addr) => {
                            let offset = *const_pool - CONSTANT_POOL_OFFSET;
                            self.constant_pool.splice(
                                (offset)..(offset + 8),
                                (*addr as u64).to_le_bytes().to_vec(),
                            );
                            self.instructions[i] = Instruction::Ld(
                                *dst,
                                RegImmAddr::Address((offset as isize - *pc as isize) as i16),
                            )
                        }
                        None => self.errmsg(format!("Label \"{}\" is undefined", label)),
                    }
                }
                Instruction::B(RegImmAddr::Unresolved(label, pc, _)) => {
                    match self.mapping.get(label) {
                        Some(addr) => {
                            self.instructions[i] = Instruction::B(RegImmAddr::Address(
                                (*addr as isize - *pc as isize) as i16,
                            ))
                        }
                        None => self.errmsg(format!("Label \"{}\" is undefined", label)),
                    }
                }
                Instruction::CBZ(reg, RegImmAddr::Unresolved(label, pc, _)) => {
                    match self.mapping.get(label) {
                        Some(addr) => {
                            self.instructions[i] = Instruction::CBZ(
                                *reg,
                                RegImmAddr::Address((*addr as isize - *pc as isize) as i16),
                            )
                        }
                        None => self.errmsg(format!("Label \"{}\" is undefined", label)),
                    }
                }
                Instruction::CBNZ(reg, RegImmAddr::Unresolved(label, pc, _)) => {
                    match self.mapping.get(label) {
                        Some(addr) => {
                            self.instructions[i] = Instruction::CBNZ(
                                *reg,
                                RegImmAddr::Address((*addr as isize - *pc as isize) as i16),
                            )
                        }
                        None => self.errmsg(format!("Label \"{}\" is undefined", label)),
                    }
                }
                _ => continue,
            }
        }
    }
}
