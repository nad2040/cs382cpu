#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Loc {
    pub line: u32,
    pub col: u32,
}

impl Loc {
    pub fn new(line: u32, col: u32) -> Self {
        Self { line, col }
    }
}

#[derive(Debug, Clone)]
pub enum SectionDirective {
    Data,
    Text,
}

#[derive(Debug, Clone)]
pub enum DataTypeDirective {
    String,
    Char,
    Byte1,
    Byte2,
    Byte4,
    Byte8,
}

#[derive(Debug, Clone)]
pub enum CommentType {
    Line,
    MultiLine,
}

#[derive(Debug, Clone)]
pub enum TokenValue {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Asr,
    Lsl,

    And,
    Orr,
    Neg,

    Ld(u8, bool),
    St(u8),

    Register(u8),
    Imm(u64),
    Char(char),
    String(String),

    Label(String),
    LabelDef(String),

    SectionDirective(SectionDirective),
    DataTypeDirective(DataTypeDirective),

    Comma,
    Colon,

    Whitespace,
    Newline,

    Eof,
}

#[derive(Debug, Clone)]
pub struct Token {
    loc: Loc,
    value: TokenValue,
}

impl Token {
    pub fn new(loc: Loc, value: TokenValue) -> Self {
        Self { loc, value }
    }
}
