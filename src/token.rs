#[derive(Hash, Debug, Clone, Copy, PartialEq, Eq)]
pub struct Loc {
    pub line: u32,
    pub col: u32,
}

// impl Loc {
//     pub fn new(line: u32, col: u32) -> Self {
//         Self { line, col }
//     }
// }

#[derive(Hash, PartialEq, Eq, Debug, Clone)]
pub enum SectionDirective {
    Data,
    Text,
}

#[derive(Hash, PartialEq, Eq, Debug, Clone)]
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

#[derive(Hash, PartialEq, Eq, Debug, Clone)]
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

    Swap,

    Ld(u8, bool), // num bytes, sign-extension
    St(u8),       // num bytes

    Halt,

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
    LBracket,
    RBracket,

    Whitespace,
    Newline,

    Eof,
}

#[derive(Hash, PartialEq, Eq, Debug, Clone)]
pub struct Token {
    pub loc: Loc,
    pub value: TokenValue,
}

impl Token {
    pub fn new(loc: Loc, value: TokenValue) -> Self {
        Self { loc, value }
    }
}
