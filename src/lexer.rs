use crate::token::{CommentType, DataTypeDirective, Loc, SectionDirective, Token, TokenValue};
use log::error;

#[derive(Debug, Clone)]
pub struct Lexer {
    source: String,
    pub tokens: Vec<Token>,
    start_idx: usize,
    start_loc: Loc,
    curr_idx: usize,
    curr_loc: Loc,
}

#[allow(dead_code)]
impl Lexer {
    pub fn new(source: String) -> Self {
        let mut l = Self {
            source,
            tokens: Vec::new(),
            start_idx: 0,
            start_loc: Loc { line: 1, col: 1 },
            curr_idx: 0,
            curr_loc: Loc { line: 1, col: 1 },
        };
        l.scan_tokens();
        l
    }

    pub fn emit(&self) {
        for token in &self.tokens {
            println!("{:?}", token);
        }
    }

    fn add_token(&mut self, token: Token) {
        self.tokens.push(token)
    }

    fn peek(&self) -> char {
        self.source.as_bytes()[self.curr_idx] as char
    }
    fn peek_n(&self, n: usize) -> &str {
        if self.is_at_end() {
            self.error(
                "End-of-file reached when reading token".to_string(),
                self.curr_loc.line,
                self.curr_loc.col,
            );
        }
        std::str::from_utf8(
            self.source.as_bytes()
                [self.curr_idx..std::cmp::min(self.curr_idx + n, self.source.len())]
                .as_ref(),
        )
        .unwrap()
    }

    fn match_str(&mut self, string: &str) -> bool {
        if string == self.peek_n(string.len()) {
            self.increment_position(string.len());
            true
        } else {
            false
        }
    }

    fn scan_tokens(&mut self) {
        while !self.is_at_end() {
            self.start_idx = self.curr_idx;
            self.start_loc = self.curr_loc;
            self.parse_token();
        }
        self.start_idx = self.curr_idx;
        self.start_loc = self.curr_loc;
        self.tokens
            .push(Token::new(self.start_loc, TokenValue::Eof));
    }

    fn parse_token(&mut self) {
        let mut c = self.peek();
        match c {
            ':' => {
                self.increment_position(1);
                self.add_token(Token::new(self.start_loc, TokenValue::Colon));
            }
            ',' => {
                self.increment_position(1);
                self.add_token(Token::new(self.start_loc, TokenValue::Comma));
            }
            '[' => {
                self.increment_position(1);
                self.add_token(Token::new(self.start_loc, TokenValue::LBracket));
            }
            ']' => {
                self.increment_position(1);
                self.add_token(Token::new(self.start_loc, TokenValue::RBracket));
            }
            c if c.is_whitespace() && c != '\n' => self.parse_whitespace(),
            '\n' => {
                self.increment_position(1);
                self.add_token(Token::new(self.start_loc, TokenValue::Newline));
            }
            '/' => {
                self.increment_position(1);
                c = self.peek();
                match c {
                    '/' => self.parse_comment(CommentType::Line),
                    '*' => self.parse_comment(CommentType::MultiLine),
                    _ => self.error(
                        "Unknown comment specifier".to_string(),
                        self.curr_loc.line,
                        self.curr_loc.col,
                    ),
                }
            }
            c if c.is_ascii_alphabetic() || c == '_' => self.parse_word(),
            '0'..='9' => self.parse_immediate(),
            '.' => self.parse_directive(),
            '\'' => self.parse_char(),
            '\"' => self.parse_string(),
            _ => self.error(
                "Unknown token".to_string(),
                self.curr_loc.line,
                self.curr_loc.col,
            ),
        }
    }

    fn parse_whitespace(&mut self) {
        let mut c = self.peek();
        while c.is_whitespace() && c != '\n' {
            self.increment_position(1);
            c = self.peek();
        }
        self.add_token(Token::new(self.start_loc, TokenValue::Whitespace));
    }

    fn parse_comment(&mut self, comment_type: CommentType) {
        match comment_type {
            CommentType::Line => {
                self.increment_position(1);
                let mut c = self.peek();
                while c != '\n' {
                    self.increment_position(1);
                    c = self.peek();
                }
            }
            CommentType::MultiLine => {
                self.increment_position(1);
                let mut end_comment = self.peek_n(2);
                while end_comment != "*/" {
                    self.increment_position(1);
                    end_comment = self.peek_n(2);
                }
                self.increment_position(2);
            }
        }
    }

    fn parse_word(&mut self) {
        let mut str = String::new();
        let mut c = self.peek();
        while c.is_ascii_alphanumeric() || c == '_' || c == '.' {
            str.push(c);
            self.increment_position(1);
            c = self.peek();
        }

        match str.to_lowercase().as_str() {
            "add" => self.add_token(Token::new(self.start_loc, TokenValue::Add)),
            "sub" => self.add_token(Token::new(self.start_loc, TokenValue::Sub)),
            "mul" => self.add_token(Token::new(self.start_loc, TokenValue::Mul)),
            "div" => self.add_token(Token::new(self.start_loc, TokenValue::Div)),
            "mod" => self.add_token(Token::new(self.start_loc, TokenValue::Mod)),
            "asr" => self.add_token(Token::new(self.start_loc, TokenValue::Asr)),
            "lsl" => self.add_token(Token::new(self.start_loc, TokenValue::Lsl)),
            "and" => self.add_token(Token::new(self.start_loc, TokenValue::And)),
            "orr" => self.add_token(Token::new(self.start_loc, TokenValue::Orr)),
            "neg" => self.add_token(Token::new(self.start_loc, TokenValue::Neg)),
            "swap" => self.add_token(Token::new(self.start_loc, TokenValue::Swap)),
            "halt" => self.add_token(Token::new(self.start_loc, TokenValue::Halt)),
            "ld1" | "ld1s" => self.add_token(Token::new(
                self.start_loc,
                TokenValue::Ld(1, str.len() == 4),
            )),
            "ld2" | "ld2s" => self.add_token(Token::new(
                self.start_loc,
                TokenValue::Ld(2, str.len() == 4),
            )),
            "ld4" | "ld4s" => self.add_token(Token::new(
                self.start_loc,
                TokenValue::Ld(4, str.len() == 4),
            )),
            "ld" | "lds" => self.add_token(Token::new(
                self.start_loc,
                TokenValue::Ld(8, str.len() == 3),
            )),
            "st1" | "st2" | "st4" | "st" => self.add_token(Token::new(
                self.start_loc,
                TokenValue::St(if str.len() == 2 {
                    8
                } else {
                    str.chars().last().unwrap().to_digit(10).unwrap() as u8
                }),
            )),
            "b" => self.add_token(Token::new(self.start_loc, TokenValue::B)),
            "cbz" => self.add_token(Token::new(self.start_loc, TokenValue::CBZ)),
            "cbnz" => self.add_token(Token::new(self.start_loc, TokenValue::CBNZ)),
            "rzr" => self.add_token(Token::new(self.start_loc, TokenValue::Register(7))),
            "r0" | "r1" | "r2" | "r3" | "r4" | "r5" | "r6" | "r7" => self.add_token(Token::new(
                self.start_loc,
                TokenValue::Register(str.chars().last().unwrap().to_digit(10).unwrap() as u8),
            )),
            _ => {
                if c == ':' {
                    self.increment_position(1);
                    self.add_token(Token::new(self.start_loc, TokenValue::LabelDef(str)))
                } else if c.is_whitespace() {
                    self.add_token(Token::new(self.start_loc, TokenValue::Label(str)))
                }
            }
        }
    }

    fn parse_immediate(&mut self) {
        use std::num::IntErrorKind;
        let header = self.peek_n(2);
        let mut num = String::new();
        match header {
            "0x" => {
                self.increment_position(2);
                let mut c = self.peek();
                while c.is_digit(16) {
                    num.push(c);
                    self.increment_position(1);
                    c = self.peek();
                }
                match u64::from_str_radix(num.as_str(), 16) {
                    Ok(num) => self.add_token(Token::new(self.start_loc, TokenValue::Imm(num))),
                    Err(e) => match e.kind() {
                        IntErrorKind::PosOverflow | IntErrorKind::NegOverflow => self.error(
                            "Hex literal exceeds 64 bits".to_string(),
                            self.start_loc.line,
                            self.start_loc.col,
                        ),
                        IntErrorKind::Empty => {
                            self.error(
                                "Incomplete hex literal".to_string(),
                                self.curr_loc.line,
                                self.curr_loc.col,
                            );
                        }
                        _ => (),
                    },
                }
            }
            "0b" => {
                self.increment_position(2);
                let mut c = self.peek();
                while c.is_digit(2) {
                    num.push(c);
                    self.increment_position(1);
                    c = self.peek();
                }
                match u64::from_str_radix(num.as_str(), 2) {
                    Ok(num) => self.add_token(Token::new(self.start_loc, TokenValue::Imm(num))),
                    Err(e) => match e.kind() {
                        IntErrorKind::PosOverflow | IntErrorKind::NegOverflow => self.error(
                            "Binary literal exceeds 64 bits".to_string(),
                            self.start_loc.line,
                            self.start_loc.col,
                        ),
                        IntErrorKind::Empty => {
                            self.error(
                                "Incomplete binary literal".to_string(),
                                self.curr_loc.line,
                                self.curr_loc.col,
                            );
                        }
                        _ => (),
                    },
                }
            }
            _ => {
                let mut c = self.peek();
                while c.is_digit(10) {
                    num.push(c);
                    self.increment_position(1);
                    c = self.peek();
                }
                match u64::from_str_radix(num.as_str(), 10) {
                    Ok(num) => self.add_token(Token::new(self.start_loc, TokenValue::Imm(num))),
                    Err(e) => match e.kind() {
                        IntErrorKind::PosOverflow | IntErrorKind::NegOverflow => self.error(
                            "Decimal literal exceeds 64 bits".to_string(),
                            self.start_loc.line,
                            self.start_loc.col,
                        ),
                        IntErrorKind::Empty => {
                            self.error(
                                "Incomplete decimal literal".to_string(),
                                self.curr_loc.line,
                                self.curr_loc.col,
                            );
                        }
                        _ => (),
                    },
                }
            }
        }
    }

    fn parse_directive(&mut self) {
        self.increment_position(1);
        if self.match_str("text") {
            self.add_token(Token::new(
                self.start_loc,
                TokenValue::SectionDirective(SectionDirective::Text),
            ))
        } else if self.match_str("data") {
            self.add_token(Token::new(
                self.start_loc,
                TokenValue::SectionDirective(SectionDirective::Data),
            ))
        } else if self.match_str("char") {
            self.add_token(Token::new(
                self.start_loc,
                TokenValue::DataTypeDirective(DataTypeDirective::Char),
            ))
        } else if self.match_str("string") {
            self.add_token(Token::new(
                self.start_loc,
                TokenValue::DataTypeDirective(DataTypeDirective::String),
            ))
        } else if self.match_str("1b") {
            self.add_token(Token::new(
                self.start_loc,
                TokenValue::DataTypeDirective(DataTypeDirective::Byte1),
            ))
        } else if self.match_str("2b") {
            self.add_token(Token::new(
                self.start_loc,
                TokenValue::DataTypeDirective(DataTypeDirective::Byte2),
            ))
        } else if self.match_str("4b") {
            self.add_token(Token::new(
                self.start_loc,
                TokenValue::DataTypeDirective(DataTypeDirective::Byte4),
            ))
        } else if self.match_str("8b") {
            self.add_token(Token::new(
                self.start_loc,
                TokenValue::DataTypeDirective(DataTypeDirective::Byte8),
            ))
        } else {
            self.error(
                "Unknown directive".to_string(),
                self.start_loc.line,
                self.start_loc.col,
            );
        }
    }

    fn parse_char(&mut self) {
        self.increment_position(1);
        let mut c = self.peek();
        match c {
            '\n' | '\r' | '\t' | '\0' | '\'' => {
                self.error(
                    "Invalid character literal".to_string(),
                    self.curr_loc.line,
                    self.curr_loc.col,
                );
            }
            '\\' => {
                self.increment_position(1);
                c = self.peek();
                match c {
                    'n' => self.add_token(Token::new(self.start_loc, TokenValue::Char('\n'))),
                    'r' => self.add_token(Token::new(self.start_loc, TokenValue::Char('\r'))),
                    't' => self.add_token(Token::new(self.start_loc, TokenValue::Char('\t'))),
                    '0' => self.add_token(Token::new(self.start_loc, TokenValue::Char('\0'))),
                    '\'' => self.add_token(Token::new(self.start_loc, TokenValue::Char('\''))),
                    _ => self.error(
                        "Invalid escape sequence".to_string(),
                        self.curr_loc.line,
                        self.curr_loc.col,
                    ),
                }
                self.increment_position(1);
            }
            c => {
                self.increment_position(1);
                self.add_token(Token::new(self.start_loc, TokenValue::Char(c)));
            }
        }
        self.increment_position(1);
    }
    fn parse_string(&mut self) {
        self.increment_position(1);
        let mut str = String::new();
        while self.peek() != '"' && !self.is_at_end() {
            let c = self.peek();
            if c == '\\' {
                if self.match_str("\\0") {
                    str.push('\0');
                } else if self.match_str("\\n") {
                    str.push('\n');
                } else if self.match_str("\\r") {
                    str.push('\r');
                } else if self.match_str("\\t") {
                    str.push('\t');
                } else if self.match_str("\\\\") {
                    str.push('\\');
                } else if self.match_str("\\\"") {
                    str.push('"');
                } else {
                    self.error(
                        "Invalid escape sequence".to_string(),
                        self.curr_loc.line,
                        self.curr_loc.col,
                    );
                }
                self.increment_position(2);
            } else {
                str.push(c);
                self.increment_position(1);
            }
        }

        if self.is_at_end() {
            self.error(
                "Unterminated string".to_string(),
                self.curr_loc.line,
                self.curr_loc.col,
            );
        }

        self.increment_position(1);
        self.add_token(Token::new(self.start_loc, TokenValue::String(str)));
    }

    fn error(&self, message: String, line: u32, col: u32) {
        error!("{} at {}:{}", message, line, col);
        std::process::exit(1);
    }

    fn is_at_end(&self) -> bool {
        self.curr_idx >= self.source.len()
    }

    fn increment_position(&mut self, n: usize) {
        for _ in 0..n {
            if self.source.chars().nth(self.curr_idx).unwrap() == '\n' {
                self.curr_idx += 1;
                self.curr_loc.line += 1;
                self.curr_loc.col = 1;
            } else {
                self.curr_idx += 1;
                self.curr_loc.col += 1;
            }
        }
    }
}
