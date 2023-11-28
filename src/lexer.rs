use crate::token::{DataTypeDirective, Loc, SectionDirective, Token, TokenValue};

#[derive(Debug, Clone)]
pub struct Lexer {
    source: String,
    tokens: Vec<Token>,
    start_idx: usize,
    start_loc: Loc,
    curr_idx: usize,
    curr_loc: Loc,
}

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

    fn add_token(&self, token: Token) {
        self.tokens.push(token)
    }

    fn peek(&self) -> char {
        self.source.as_bytes()[self.curr_idx] as char
    }
    fn peek_n(&self, n: usize) -> &str {
        std::str::from_utf8(
            self.source.as_bytes()
                [self.curr_idx..std::cmp::min(self.curr_idx + n, self.source.len())]
                .as_ref(),
        )
        .unwrap()
    }

    fn match_str(&self, string: &str) -> bool {
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

    fn parse_token(&self) {
        let c = self.peek();
        match c {
            ' ' => self.parse_whitespace(),
            '.' => self.parse_directive(),
            ',' => {
                self.increment_position(1);
                self.add_token(Token::new(self.start_loc, TokenValue::Comma));
            }
            '\'' => self.parse_char(),
            '\"' => self.parse_string(),
        }
    }

    fn parse_whitespace(&self) {
        let mut c = self.peek();
        while c.is_whitespace() && c != '\n' {
            self.increment_position(1);
            c = self.peek();
        }
        self.add_token(Token::new(self.start_loc, TokenValue::Whitespace));
    }

    fn parse_directive(&self) {
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
                "Error: unknown directive".to_string(),
                self.start_loc.line,
                self.start_loc.col,
            );
        }
    }

    fn parse_char(&self) {
        self.increment_position(1);
        let mut c = self.peek();
        match c {
            '\n' | '\r' | '\t' | '\0' | '\'' => {
                self.error(
                    "Error: Invalid character literal".to_string(),
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
                        "Error: invalid escape sequence".to_string(),
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
    fn parse_string(&self) {
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
                        "Error: Invalid escape sequence".to_string(),
                        self.curr_loc.line,
                        self.curr_loc.col,
                    );
                }
            } else {
                str.push(c);
            }
        }

        if self.is_at_end() {
            self.error(
                "Error: Unterminated string".to_string(),
                self.curr_loc.line,
                self.curr_loc.col,
            );
        }

        self.increment_position(1);
        self.add_token(Token::new(self.start_loc, TokenValue::String(str)));
    }

    fn error(&self, message: String, line: u32, col: u32) {
        panic!("{} at {}:{}", message, line, col);
    }

    fn is_at_end(&self) -> bool {
        self.curr_idx >= self.source.len()
    }

    fn increment_position(&mut self, n: usize) {
        for i in 0..n {
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
